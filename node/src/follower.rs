use std::sync::Arc;
use std::time::Duration;

use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::ledger::Ledger;
use common::{ClientResponse, ReplicationMessage, wire};

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

    pub async fn run(&self, leader_follower_addr: &str){
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
