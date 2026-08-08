use std::collections::HashMap;
use common::{Cents, Event};

#[derive(Debug, Default)]
pub struct StateMachine{
    balalnce: HashMap<String, Cents>
}

impl StateMachine{
    pub fn new() -> Self{
        Self{
            balalnce: HashMap::new()
        }
    }

    pub fn apply(&mut self, event: &Event){
        match event{
            Event::Deposite { account, amount }=>{
                *self.balalnce.entry(account.clone()).or_insert(0) += amount;
            }
            Event::Withdraw { account, amount }=>{
                *self.balalnce.entry(account.clone()).or_insert(0) -= amount;
            }
            Event::TransferDebit { from, amount, .. }=>{
                *self.balalnce.entry(from.clone()).or_insert(0) -= amount;
            }
            Event::TransferCredit { to, amount, .. }=>{
                *self.balalnce.entry(to.clone()).or_insert(0) += amount;
            }
        }
    }

    pub fn balance(&self, acccount: &str) -> Cents{
        *self.balalnce.get(acccount).unwrap_or(&0)
    }
}