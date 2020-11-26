use crate::error::Result;
use crate::logger::LogManager;
use crate::memtable::{MemTable, MemTableMut};
use crate::session::{Session, SessionFactory};
use crate::skip_list::RandomLevelGenerator;
use crate::Comparator;
use std::mem::replace;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

#[allow(dead_code)]
pub struct DBCore<C: Comparator> {
    session_factory: SessionFactory,
    memtable_log_number: AtomicU64,
    mutable_memtable: RwLock<MemTableMut<C>>,
    immutable_memtables: RwLock<Vec<MemTable<C>>>,
    log_manager: LogManager,
}

unsafe impl<C: Comparator> Sync for DBCore<C> {}

#[allow(dead_code)]
impl<C: Comparator> DBCore<C> {
    pub fn get_session(&self, core_arc: Arc<DBCore<C>>) -> Session<C> {
        self.session_factory.get_session(core_arc)
    }

    pub fn drop_session(&self, sequence: u64) {
        self.session_factory.drop_sequence(sequence)
    }

    fn create_memtable(&self) -> MemTableMut<C> {
        // todo: get level generator and block_size from config
        MemTableMut::new(
            self.memtable_log_number.fetch_add(1, Ordering::SeqCst),
            Box::new(RandomLevelGenerator::new(10, 0.1)),
            1024 * 4,
        )
    }

    fn renew_memtable(&self) -> Result<()> {
        let mut guard = self.mutable_memtable.write().unwrap();
        let old = replace(&mut *guard, self.create_memtable());
        let immutable = old.freeze();
        self.log_manager.freeze_current_file(guard.log_number())?;

        self.immutable_memtables.write().unwrap().push(immutable);

        Ok(())
    }
}
