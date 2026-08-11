use std::sync::Arc;

use common::ClientRequest::Withdraw;
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
                            if let Err(e) = leader.server_client(socket).await{
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
                            if let Err(e) = leader.server_client(socket).await{
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

    async fn server_client(&self, mut socket:TcpStream) -> std::io::Result<()>{
        loop{
            let request: ClientRequest = wire::read_message(&mut socket).await?;
            let response = self.handle_request(request).await;
            wire::write_message(&mut socket, &response).await?;
        }
    }


    async fn handle_request(&self, request:ClientRequest) -> ClientResponse{
        match request{
            ClientRequest::Deposite { account, amount }=>{
                println!("account: {account}, amount: {amount}");
                ClientResponse::Balance { account, amount, as_of_offset: 0 }
            }
            ClientRequest::Withdraw { account, amount }=>{
                println!("account: {account}, amount: {amount}");
                ClientResponse::Balance { account, amount, as_of_offset: 1 }
            }
            ClientRequest::Transfer { from, to, amount }=>{
                println!("from: {from}, amount: {amount}");
                ClientResponse::Balance { account: "abc".into(), amount, as_of_offset:  2}
            }
            ClientRequest::GetBalance { account, read_after }=>{
                ClientResponse::Balance { account, amount: 0, as_of_offset: 3 }
            }
        }
    }
}