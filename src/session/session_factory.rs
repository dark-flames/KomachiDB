use crate::core::DBCore;
use crate::session::session_handler::Session;
use crate::Comparator;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

pub struct SessionFactory {
    sequence: AtomicU64,
    in_use_sequence: RwLock<HashSet<u64>>,
}

unsafe impl Sync for SessionFactory {}

#[allow(dead_code)]
impl SessionFactory {
    pub fn new(sequence: u64) -> Self {
        SessionFactory {
            sequence: AtomicU64::new(sequence),
            in_use_sequence: RwLock::new(HashSet::new()),
        }
    }

    pub fn get_session<C: Comparator>(&self, core: Arc<DBCore<C>>) -> Session<C> {
        let mut in_use_set = self.in_use_sequence.write().unwrap();
        let sequence = self.sequence.fetch_add(1, Ordering::SeqCst);
        in_use_set.insert(sequence);

        Session::new(sequence, core)
    }

    pub fn drop_sequence(&self, sequence: u64) {
        let mut in_use_set = self.in_use_sequence.write().unwrap();

        in_use_set.remove(&sequence);
    }

    pub fn sequence_in_use(&self, sequence: u64) -> bool {
        let in_use_set = self.in_use_sequence.read().unwrap();

        in_use_set.contains(&sequence)
    }
}
