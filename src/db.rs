use crate::core::DBCore;
use crate::session::Session;
use crate::Comparator;
use std::sync::Arc;

pub struct KomachiDB<C: Comparator> {
    core: Arc<DBCore<C>>,
}

impl<C: Comparator> KomachiDB<C> {
    pub fn new_session(&self) -> Session<C> {
        self.core.get_session(self.core.clone())
    }
}
