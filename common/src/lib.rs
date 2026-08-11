mod event;
mod protocol;
pub mod wire;
pub use event::Cents;
pub use event::Event;
pub use event::LogEntry;
pub use protocol::{ClientRequest, ClientResponse, ReplicationMessage};
pub use wire::{read_message, write_message};

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
