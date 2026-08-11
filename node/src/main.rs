mod leader;
mod log;
mod state;
use common::Event;
use log::AppendOnlyLog;
use common::wire::{read_message, write_message};
fn main() {
    let mut log = AppendOnlyLog::new();
    log.append(Event::Deposite {
        account: "angshu".to_string(),
        amount: 500,
    });
}
