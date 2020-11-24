use crate::session::{Session, SessionFactory};
use std::sync::Arc;

pub struct DBCore {
    session_factory: SessionFactory,
}

unsafe impl Sync for DBCore {}

impl DBCore {
    pub fn get_session(&self, core_arc: Arc<DBCore>) -> Session {
        self.session_factory.get_session(core_arc)
    }

    pub fn drop_session(&self, sequence: u64) {
        self.session_factory.drop_sequence(sequence)
    }
}
