use common::{Cents, Event};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct StateMachine {
    balance: HashMap<String, Cents>,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            balance: HashMap::new(),
        }
    }

    pub fn apply(&mut self, event: &Event) {
        match event {
            Event::Deposit { account, amount } => {
                *self.balance.entry(account.clone()).or_insert(0) += amount;
            }
            Event::Withdraw { account, amount } => {
                *self.balance.entry(account.clone()).or_insert(0) -= amount;
            }
            Event::TransferDebit { from, amount, .. } => {
                *self.balance.entry(from.clone()).or_insert(0) -= amount;
            }
            Event::TransferCredit { to, amount, .. } => {
                *self.balance.entry(to.clone()).or_insert(0) += amount;
            }
        }
    }

    pub fn balance(&self, acccount: &str) -> Cents {
        *self.balance.get(acccount).unwrap_or(&0)
    }
}