use serde::{Serialize, Deserialize};
use crate::event::{Cents, LogEntry};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientRequest{
    Deposite { account: String, amount: Cents},
    Withdraw {account: String, amount: Cents},
    Transfer {from: String, to: String, amount: Cents},
    GetBalance {account: String, read_after: Option<u64>}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientResponse{
    WriteAck {offset: u64},
    Balance{account: String, amount: Cents, as_of_offset: u64},
    NotLeader {leader_addr: Option<String>},
    Error{message: String}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplicationMessage{
    Subscribe{from_offset: u64},
    Entry(LogEntry),
    Heartbeat{leader_offset: u64}
}