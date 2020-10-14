use crate::interface::Key;
use crate::memory_pool::MemoryPool;
use crate::skip_list::SkipList;
use format::{SequenceNumber, ValueType};

mod format;

#[allow(dead_code)]
pub struct MemTable<K: Key> {
    pool: MemoryPool,
    skip_list: SkipList<K>,
}

#[allow(dead_code)]
impl<K: Key> MemTable<K> {
    pub fn add(_sequence_number: SequenceNumber, _value_type: ValueType, _key: K, _value: &[u8]) {}
}
