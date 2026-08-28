use std::sync::Arc;
use std::time::Duration;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use crate::client_handler::{self, RequestHandler};
use crate::ledger::Ledger;
use common::{ClientRequest, ClientResponse, ReplicationMessage, wire};

#[derive(Clone)]
pub struct Follower {
    ledger: Arc<Mutex<Ledger>>,
    leader_client_addr: String,
}

impl Follower {
    pub fn new(leader_client_addr: String) -> Self {
        Self {
            ledger: Arc::new(Mutex::new(Ledger::new())),
            leader_client_addr,
        }
    }

    pub async fn run(&self, client_addr: &str, leader_follower_addr: &str) -> std::io::Result<()>{
        let client_listener = TcpListener::bind(client_addr).await?;
        println!("follower: clients on {client_addr}, replicating from {leader_follower_addr}");

        let client_follower = self.clone();

        let clients = tokio::spawn(async move {
            loop{
                match client_listener.accept().await{
                    Ok((socket, addr)) => {
                        let follower = client_follower.clone();
                        tokio::spawn(async move {
                            if let Err(e) = client_handler::serve_client(socket, follower).await{
                                eprintln!("client {addr} disconnected: {e}");
                            }
                        });
                    }

                    Err(e) => eprintln!("client accept error: {e}")
                }
            }
        });

        let replication_follower = self.clone();
        let leader_addr = leader_follower_addr.to_string();
        let replication = tokio::spawn(async move{
            replication_follower.replicate_forever(&leader_addr).await;
        });

        let _ = tokio::join!(clients, replication);
        Ok(())
    }

    async fn replicate_forever(&self, leader_follower_addr: &str){
        loop{
            if let Err(e) = self.replication_from(leader_follower_addr).await{
                eprintln!("follower: lost connection to leader ({e}), retrying in 1s");
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    async fn replication_from(&self, leader_follower_addr: &str) -> std::io::Result<()>{
        let mut socket = TcpStream::connect(leader_follower_addr).await?;

        let from_offset = {
            let ledger = self.ledger.lock().await;
            ledger.log.last_offset()
        };

        wire::write_message(&mut socket, &ReplicationMessage::Subscribe { from_offset }).await?;
        println!("follower: subscribed to {leader_follower_addr} from offset {from_offset}");

        loop{
            let message: ReplicationMessage = wire::read_message(&mut socket).await?;
            match message{
                ReplicationMessage::Entry(entry)=>{
                    let mut ledger = self.ledger.lock().await;
                    ledger.append_replicated(entry);
                }
                ReplicationMessage::Heartbeat { leader_offset }=>{
                    let ledger = self.ledger.lock().await;
                    let behind = leader_offset.saturating_sub(ledger.log.last_offset());
                    if behind > 0{
                        println!("follower: {behind} entries behind the leader");
                    }
                }

                ReplicationMessage::Subscribe { .. }=> {}
            }
        }
    }

    pub async fn balance(&self, account: &str, read_after: Option<u64>) -> ClientResponse{
        if let Some(needed) = read_after{
            loop{
                {
                    let ledger = self.ledger.lock().await;
                    if ledger.log.last_offset() >= needed{
                        break;
                    }
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        }

        let ledger = self.ledger.lock().await;
        ClientResponse::Balance { 
            amount: ledger.state.balance(account), 
             as_of_offset: ledger.log.last_offset(),
             account: account.to_string()
            }
    }

    pub fn reject_write(&self) -> ClientResponse{
        ClientResponse::NotLeader { 
            leader_addr: Some(self.leader_client_addr.clone())
         }
    }
}

impl RequestHandler for Follower{
    async fn handle_request(&self, request: ClientRequest) -> ClientResponse {
        match request {
            ClientRequest::GetBalance { account, read_after } =>{
                self.balance(&account, read_after).await
            }
            ClientRequest::Deposite { .. }
            | ClientRequest::Withdraw { .. }
            | ClientRequest::Transfer { .. } => {
                self.reject_write()
            }
        }
    }
}