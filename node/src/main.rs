mod leader;
mod log;
mod state;
mod follower;
mod ledger;
mod client_handler;
use common::Event;
use log::AppendOnlyLog;
use common::wire::{read_message, write_message};
use leader::Leader;
fn main() {
    
}
