use common::{LogEntry};

#[derive(Debug, Default)]
pub struct AppendOnlyLog{
    entries: Vec<LogEntry>
}

impl AppendOnlyLog{
    pub fn new() -> Self{
        Self { entries: Vec::new() }
    }

    pub fn from_entries(entries: Vec<LogEntry>)-> Self{
        Self {entries}
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