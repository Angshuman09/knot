use std::sync::Arc;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, broadcast};

use common::{
    ClientRequest, ClientResponse, Event, LogEntry, ReplicationMessage, wire
};

use crate::log::AppendOnlyLog;
use crate::state::StateMachine;


struct Ledger{
    log: AppendOnlyLog,
    state: StateMachine
}

#[derive(Clone)]
pub struct Leader{
    ledger: Arc<Mutex<Ledger>>,
    new_entries: broadcast::Sender<LogEntry>
}

impl Leader{
    pub fn new() -> Self{
        let (new_entries, _unused_reciever) = broadcast::channel(1024);
        Self { ledger: Arc::new(Mutex::new(
            Ledger{
                log: AppendOnlyLog::new(),
                state: StateMachine::new()
            }
        )), new_entries }
    }

    pub async fn run(self, client_addr: &str, follower_addr:&str) -> std::io::Result<()>{
        let client_listener = TcpListener::bind(client_addr).await?;
        let follower_listener = TcpListener::bind(follower_addr).await?;
        println!("leader: clients on {client_addr}, followers on {follower_addr}");

        let client_leader = self.clone();
        let clients = tokio::spawn(async move{
            loop{
                match client_listener.accept().await{
                    Ok((socket, addr))=>{
                        let leader = client_leader.clone();
                        tokio::spawn(async move{
                            if let Err(e) = leader.serve_client(socket).await{
                                eprint!("client {addr} disconnected: {e}");
                            }
                        });
                    }
                    Err(e) => eprint!("client accept error: {e}")
                }
            }
        });

        let follower_leader = self.clone();
        let followers = tokio::spawn(async move{
            loop{
                match follower_listener.accept().await{
                    Ok((socket, addr)) => {
                        let leader = follower_leader.clone();
                        tokio::spawn(async move{
                            if let Err(e) = leader.serve_follower(socket).await{
                                eprint!("follower {addr} disconnected: {e}");
                            }
                        });
                    }
                    Err(e) => eprint!("follower accept error: {e}")
                }
            }
        });

        let _ = tokio::join!(clients, followers);
        Ok(())
    }

    async fn serve_client(&self, mut socket:TcpStream) -> std::io::Result<()>{
        loop{
            let request: ClientRequest = wire::read_message(&mut socket).await?;
            let response = self.handle_request(request).await;
            wire::write_message(&mut socket, &response).await?;
        }
    }


    async fn handle_request(&self, request:ClientRequest) -> ClientResponse{
        match request{
            ClientRequest::Deposite { account, amount }=>{
                self.commit(Event::Deposite { account, amount }).await
            }
            ClientRequest::Withdraw { account, amount }=>{
                let mut ledger = self.ledger.lock().await;
                if ledger.state.balance(&account) < amount{
                    return ClientResponse::Error{
                        message: format!("insufficient funds in {account}")
                    };
                }

                let entry = Self::append_locked(&mut ledger, Event::Withdraw { account, amount });

                drop(ledger);

                let _ = self.new_entries.send(entry.clone());
                ClientResponse::WriteAck { offset: entry.offset}
            }
            ClientRequest::Transfer { from, to, amount }=>{
                let mut ledger = self.ledger.lock().await;
                if ledger.state.balance(&from) < amount {
                    return ClientResponse::Error{
                        message: format!("insufficent balance in {from}")
                    }
                }

                let transfer_id = ledger.log.last_offset() + 1;
                let debit = Self::append_locked(
                    &mut ledger,
                     Event::TransferDebit { transfer_id, from, amount },
                    );

                    let credit = Self::append_locked(
                        &mut ledger,
                        Event::TransferCredit { transfer_id, to, amount }
                    );

                    drop(ledger);
                    
                    let _ = self.new_entries.send(debit);
                    let _ = self.new_entries.send(credit.clone());
                    ClientResponse::WriteAck { offset: credit.offset }
            }
            ClientRequest::GetBalance { account, read_after: _ }=>{
                let ledger = self.ledger.lock().await;
                let amount = ledger.state.balance(&account);
                ClientResponse::Balance { 
                    account,
                    amount,
                    as_of_offset: ledger.log.last_offset() 
                }
            }
        }
    }

    async fn commit(&self, event: Event) -> ClientResponse{
        let mut ledger = self.ledger.lock().await;
        let entry = Self::append_locked(&mut ledger, event);
        drop(ledger);
        let _ = self.new_entries.send(entry.clone());
        ClientResponse::WriteAck { offset: entry.offset }
    }

    fn append_locked(ledger: &mut Ledger, event: Event) -> LogEntry{
        let entry = ledger.log.append(event);
        ledger.state.apply(&entry.event);
        entry
    }

    async fn serve_follower(&self, mut socket: TcpStream) -> std::io::Result<()>{
        Ok(())
    }
}