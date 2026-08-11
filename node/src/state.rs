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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn state_machine_applies_deposit_and_withdraw() {
        let mut state = StateMachine::new();

        state.apply(&Event::Deposit {
            account: "alice".into(),
            amount: 100,
        });

        state.apply(&Event::Withdraw {
            account: "alice".into(),
            amount: 30,
        });

        assert_eq!(state.balance("alice"), 70);
    }

    #[test]
    fn state_machine_applies_transfer() {
        let mut state = StateMachine::new();

        state.apply(&Event::Deposit {
            account: "alice".into(),
            amount: 100,
        });

        state.apply(&Event::TransferDebit {
            transfer_id: 1,
            from: "alice".into(),
            amount: 40,
        });

        state.apply(&Event::TransferCredit {
            transfer_id: 1,
            to: "bob".into(),
            amount: 40,
        });

        assert_eq!(state.balance("alice"), 60);
        assert_eq!(state.balance("bob"), 40);
    }
}