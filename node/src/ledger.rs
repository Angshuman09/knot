use crate::{log::AppendOnlyLog, storage::StorageEngine};
use crate::state::StateMachine;
use common::{Event, LogEntry};
use std::path::Path;

pub struct Ledger {
    pub log: AppendOnlyLog,
    pub state: StateMachine,
    pub storage: StorageEngine,
}

impl Ledger {
    pub fn open(data_dir: impl AsRef<Path>) -> std::io::Result<Self>{
        let storage = StorageEngine::open(data_dir)?;
        let recovered = storage.recover_entries()?;

        let mut state = StateMachine::new();
        for entry in &recovered{
            state.apply(&entry.event);
        }

        let log = AppendOnlyLog::from_entries(recovered);
        Ok(Self{
            log,
            state,
            storage
        })
    }
    
    pub fn append(&mut self, event: Event)-> std::io::Result<LogEntry>{
        let offset = self.log.last_offset() + 1;
        let entry = LogEntry{ offset, event};

        self.storage.append_entry(&entry)?;

        self.log.append_replicated(entry.clone());
        self.state.apply(&entry.event);

        Ok(entry)
    }

    pub fn append_replicated(&mut self, entry: LogEntry) -> std::io::Result<()>{
        self.storage.append_entry(&entry)?;

        self.log.append_replicated(entry.clone());
        self.state.apply(&entry.event);

        Ok(())
    }
}
