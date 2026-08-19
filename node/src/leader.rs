use std::sync::Arc;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, broadcast};

use crate::client_handler::{self, RequestHandler};
use crate::ledger::Ledger;
use crate::log::AppendOnlyLog;
use crate::state::StateMachine;
use common::{ClientRequest, ClientResponse, Event, LogEntry, ReplicationMessage, wire};

#[derive(Clone)]
pub struct Leader {
    ledger: Arc<Mutex<Ledger>>,
    new_entries: broadcast::Sender<LogEntry>,
}

impl Leader {
    pub fn new() -> Self {
        let (new_entries, _unused_reciever) = broadcast::channel(1024);
        Self {
            ledger: Arc::new(Mutex::new(Ledger {
                log: AppendOnlyLog::new(),
                state: StateMachine::new(),
            })),
            new_entries,
        }
    }

    pub async fn run(self, client_addr: &str, follower_addr: &str) -> std::io::Result<()> {
        let client_listener = TcpListener::bind(client_addr).await?;
        let follower_listener = TcpListener::bind(follower_addr).await?;
        println!("leader: clients on {client_addr}, followers on {follower_addr}");

        let client_leader = self.clone();
        let clients = tokio::spawn(async move {
            loop {
                match client_listener.accept().await {
                    Ok((socket, addr)) => {
                        let leader = client_leader.clone();
                        tokio::spawn(async move {
                            if let Err(e) = client_handler::serve_client(socket, leader).await {
                                eprint!("client {addr} disconnected: {e}");
                            }
                        });
                    }
                    Err(e) => eprint!("client accept error: {e}"),
                }
            }
        });

        let follower_leader = self.clone();
        let followers = tokio::spawn(async move {
            loop {
                match follower_listener.accept().await {
                    Ok((socket, addr)) => {
                        let leader = follower_leader.clone();
                        tokio::spawn(async move {
                            if let Err(e) = leader.serve_follower(socket).await {
                                eprint!("follower {addr} disconnected: {e}");
                            }
                        });
                    }
                    Err(e) => eprint!("follower accept error: {e}"),
                }
            }
        });

        let _ = tokio::join!(clients, followers);
        Ok(())
    }

    async fn commit(&self, event: Event) -> ClientResponse {
        let mut ledger = self.ledger.lock().await;
        let entry = ledger.append(event);
        drop(ledger);
        let _ = self.new_entries.send(entry.clone());
        ClientResponse::WriteAck {
            offset: entry.offset,
        }
    }

    async fn serve_follower(&self, mut socket: TcpStream) -> std::io::Result<()> {
        let ReplicationMessage::Subscribe { from_offset } = wire::read_message(&mut socket).await?
        else {
            return Ok(());
        };

        let (backlog, mut live) = {
            let ledger = self.ledger.lock().await;
            let backlog: Vec<LogEntry> = ledger.log.entries_after(from_offset).to_vec();
            (backlog, self.new_entries.subscribe())
        };

        for entry in backlog {
            wire::write_message(&mut socket, &ReplicationMessage::Entry(entry)).await?;
        }

        loop {
            match live.recv().await {
                Ok(entry) => {
                    wire::write_message(&mut socket, &ReplicationMessage::Entry(entry)).await?;
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    eprintln!(
                        "follower fell too far behind (missed {skipped} entries):
                        it should reconnect and resubscribe from its last known offset"
                    );
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
        Ok(())
    }
}

impl RequestHandler for Leader {
    async fn handle_request(&self, request: ClientRequest) -> ClientResponse {
        match request {
            ClientRequest::Deposite { account, amount } => {
                self.commit(Event::Deposit { account, amount }).await
            }
            ClientRequest::Withdraw { account, amount } => {
                let mut ledger = self.ledger.lock().await;
                if ledger.state.balance(&account) < amount {
                    return ClientResponse::Error {
                        message: format!("insufficinet funds in {account}"),
                    };
                }

                let entry = ledger.append(Event::Withdraw { account, amount });
                drop(ledger);
                let _ = self.new_entries.send(entry.clone());
                ClientResponse::WriteAck {
                    offset: entry.offset,
                }
            }
            ClientRequest::Transfer { from, to, amount } => {
                let mut ledger = self.ledger.lock().await;
                if ledger.state.balance(&from) < amount {
                    return ClientResponse::Error {
                        message: format!("insufficient funds in {from}"),
                    };
                }

                let tranfer_id = ledger.log.last_offset() + 1;
                let debit = ledger.append(Event::TransferDebit {
                    transfer_id,
                    from,
                    amount,
                });

                let credit = ledger.append(Event::TransferCredit {
                    transfer_id,
                    to,
                    amount,
                });

                drop(ledger);

                let _ = self.new_entries.send(debit);
                let _ = self.new_entries.send(credit.clone());
                ClientResponse::WriteAck {
                    offset: credit.offset,
                }
            }

            ClientRequest::GetBalance {
                account,
                read_after: _,
            } => {
                let ledger = self.ledger.lock().await;
                ClientResponse::Balance {
                    account,
                    amount: ledger.state.balance(&account),
                    as_of_offset: ledger.log.last_offset(),
                }
            }
        }
    }
}
