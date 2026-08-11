use serde::{Deserialize, Serialize};

pub type Cents = i64;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Event{
    Deposit {account: String, amount: Cents },
    Withdraw{account: String, amount: Cents},
    TransferDebit{transfer_id: u64, from: String, amount: Cents},
    TransferCredit{transfer_id: u64, to: String, amount: Cents}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogEntry{
    pub offset: u64, 
    pub event: Event
}