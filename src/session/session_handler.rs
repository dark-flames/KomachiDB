use crate::core::DBCore;
use std::sync::Arc;

pub struct Session {
    sequence_number: u64,
    core: Arc<DBCore>,
}

impl Session {
    pub fn new(sequence_number: u64, core: Arc<DBCore>) -> Self {
        Session {
            sequence_number,
            core,
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.core.drop_session(self.sequence_number);
    }
}
