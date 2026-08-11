mod leader;
mod log;
mod state;
use common::Event;
use log::AppendOnlyLog;
use common::wire::{read_message, write_message};
use leader::Leader;
fn main() {
    let mut log = AppendOnlyLog::new();
    log.append(Event::Deposit {
        account: "angshu".to_string(),
        amount: 500,
    });
}
