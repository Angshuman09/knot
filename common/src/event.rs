use serde::{Deserialize, Serialize};

pub type Cents = i64;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Event{
    Deposite {account: String, amount: Cent },
    Withdraw{account: String, amount: Cent},
    TransferDebit{transfer_id: u64, from: String, amount: Cent},
    TransferCredit{transfer_id: u64, to: String, amount: Cent}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogEntry{
    pub offset: u64, 
    pub event: Event
}