use crate::memtable::internal_key::{InternalKey, InternalKeyComparator};
use crate::skip_list::{Comparator, LevelGenerator, SkipList};
use bytes::Bytes;

#[allow(dead_code)]
pub struct MemTableMut<C: Comparator> {
    skip_list: SkipList<InternalKeyComparator<C>>,
}

#[allow(dead_code)]
impl<C: Comparator> MemTableMut<C> {
    pub fn new(level_generator: Box<dyn LevelGenerator>, block_size: usize) -> Self {
        MemTableMut {
            skip_list: SkipList::new(level_generator, block_size),
        }
    }

    pub fn add(&self, key: InternalKey, value: Bytes) {
        self.skip_list.insert(key.into(), value);
    }

    pub fn seek_by_internal_key(&self, key: &InternalKey) -> Option<&[u8]> {
        let mut visitor = self.skip_list.visitor();
        visitor.seek(key.as_bytes().as_ref());

        visitor.value()
    }

    pub fn memory_usage(&self) -> usize {
        self.skip_list.memory_usage()
    }
}
