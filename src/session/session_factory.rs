use crate::core::DBCore;
use crate::session::session_handler::Session;
use std::cell::UnsafeCell;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

pub struct SessionFactory {
    sequence: AtomicU64,
    in_use_sequence: RwLock<UnsafeCell<HashSet<u64>>>,
}

unsafe impl Sync for SessionFactory {}

#[allow(dead_code)]
impl SessionFactory {
    pub fn new(sequence: u64) -> Self {
        SessionFactory {
            sequence: AtomicU64::new(sequence),
            in_use_sequence: RwLock::new(UnsafeCell::new(HashSet::new())),
        }
    }

    pub fn get_session(&self, core: Arc<DBCore>) -> Session {
        let in_use_cell = self.in_use_sequence.write().unwrap();
        let in_use_set = unsafe { in_use_cell.get().as_mut().unwrap() };
        let sequence = self.sequence.fetch_add(1, Ordering::SeqCst);
        in_use_set.insert(sequence);

        Session::new(sequence, core)
    }

    pub fn drop_sequence(&self, sequence: u64) {
        let in_use_cell = self.in_use_sequence.write().unwrap();
        let in_use_set = unsafe { in_use_cell.get().as_mut().unwrap() };

        in_use_set.remove(&sequence);
    }

    pub fn sequence_in_use(&self, sequence: u64) -> bool {
        let in_use_cell = self.in_use_sequence.read().unwrap();
        let in_use_set = unsafe { in_use_cell.get().as_ref().unwrap() };

        in_use_set.contains(&sequence)
    }
}
