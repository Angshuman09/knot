mod log;
mod state;
use common::Event;
use log::AppendOnlyLog;
fn main() {
    let mut log = AppendOnlyLog::new();
    log.append(Event::Deposite {
        account: "angshu".to_string(),
        amount: 500,
    });
}
