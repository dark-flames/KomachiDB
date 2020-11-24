use crate::core::DBCore;
use crate::Comparator;
use std::sync::Arc;

pub struct Session<C: Comparator> {
    sequence_number: u64,
    core: Arc<DBCore<C>>,
}

impl<C: Comparator> Session<C> {
    pub fn new(sequence_number: u64, core: Arc<DBCore<C>>) -> Self {
        Session {
            sequence_number,
            core,
        }
    }
}

impl<C: Comparator> Drop for Session<C> {
    fn drop(&mut self) {
        self.core.drop_session(self.sequence_number);
    }
}
