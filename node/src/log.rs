use common::{Event, LogEntry};

#[derive(Debug, Default)]
pub struct AppendOnlyLog{
    entries: Vec<LogEntry>
}

impl AppendOnlyLog{
    pub fn new() -> Self{
        Self { entries: Vec::new() }
    }

    pub fn append(&mut self, event: Event)-> LogEntry{
        let offset = self.entries.len() as u64 + 1;
        let entry = LogEntry{offset , event};
        self.entries.push(entry.clone());
        entry
    }

    pub fn append_replicated(&mut self, entry: LogEntry){
        let expected = self.entries.len() as u64 + 1;
        assert_eq!(entry.offset, expected, "replication gap: expected offset{expected}, got {}", entry.offset);
        self.entries.push(entry);
    }

    pub fn last_offset(&self)-> u64{
        self.entries.len() as u64
    }

    pub fn entries_after(&self, from_offset: u64)-> &[LogEntry]{
        let start = from_offset as usize;
        if start >= self.entries.len(){
            &[]
        }else{
            &self.entries[start..]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_assigns_incrementing_offsets() {
        let mut log = AppendOnlyLog::new();

        let first = log.append(Event::Deposit {
            account: "alice".into(),
            amount: 100,
        });

        let second = log.append(Event::Deposit {
            account: "bob".into(),
            amount: 50,
        });

        assert_eq!(first.offset, 1);
        assert_eq!(second.offset, 2);
        assert_eq!(log.last_offset(), 2);
    }
}