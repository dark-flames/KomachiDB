use crate::core::DBCore;
use crate::session::Session;
use std::sync::Arc;

pub struct KomachiDB {
    core: Arc<DBCore>,
}

impl KomachiDB {
    pub fn new_session(&self) -> Session {
        self.core.get_session(self.core.clone())
    }
}
