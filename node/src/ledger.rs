use crate::log::AppendOnlyLog;
use crate::state::StateMachine;
use common::{Event, LogEntry};

#[derive(Default)]
pub struct Ledger {
    pub log: AppendOnlyLog,
    pub state: StateMachine,
}

impl Ledger {
    pub fn new() -> Self {
        Self {
            log: AppendOnlyLog::new(),
            state: StateMachine::new(),
        }
    }

    pub fn append(&mut self, event: Event)-> LogEntry{
        let entry = self.log.append(event);
        self.state.apply(&entry.event);
        entry
    }

    pub fn append_replicated(&mut self, entry: LogEntry){
        self.log.append_replicated(entry.clone());
        self.state.apply(&entry.event);
    }
}
