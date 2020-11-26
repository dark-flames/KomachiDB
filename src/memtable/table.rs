use crate::error::Result;
use crate::format::{ValueTag, ValueType};
use crate::logger::LogNumber;
use crate::memtable::internal_key::{InternalKey, InternalKeyComparator};
use crate::skip_list::{LevelGenerator, SkipList, SkipListIterator};
use crate::Comparator;
use bytes::Bytes;
use std::cmp::Ordering;

#[allow(dead_code)]
pub struct MemTableMut<C: Comparator> {
    log_number: LogNumber,
    skip_list: SkipList<InternalKeyComparator<C>>,
}

#[allow(dead_code)]
impl<C: Comparator> MemTableMut<C> {
    pub fn new(
        log_number: LogNumber,
        level_generator: Box<dyn LevelGenerator>,
        block_size: usize,
    ) -> Self {
        MemTableMut {
            log_number,
            skip_list: SkipList::new(level_generator, block_size),
        }
    }

    pub fn log_number(&self) -> LogNumber {
        self.log_number
    }

    pub fn add(&self, key: InternalKey, value: Bytes) {
        self.skip_list.insert(key.into(), value);
    }

    pub fn seek_by_internal_key(&self, key: &InternalKey) -> Option<&[u8]> {
        let mut visitor = self.skip_list.visitor();
        visitor.seek(key.as_bytes().as_ref());

        visitor.value()
    }

    pub fn seek_by_key_and_sequence(
        &self,
        key: &Bytes,
        sequence: u64,
    ) -> Result<Option<(ValueTag, &[u8])>> {
        let mut visitor = self.skip_list.visitor();
        let search_key = InternalKey::new(key.clone(), ValueTag::new(sequence, ValueType::Value)?);

        visitor.seek_less_or_equal(search_key.as_bytes().as_ref());

        Ok(match visitor.key().map(|k| InternalKey::split(k)) {
            None => None,
            Some((tag, result_key)) => {
                if C::compare(key.as_ref(), result_key) == Ordering::Equal {
                    visitor.value().map(|v| (tag, v))
                } else {
                    None
                }
            }
        })
    }

    pub fn memory_usage(&self) -> usize {
        self.skip_list.memory_usage()
    }

    pub fn iter(&self) -> SkipListIterator<InternalKeyComparator<C>> {
        self.skip_list.iter()
    }

    pub fn freeze(self) -> MemTable<C> {
        self.into()
    }
}

unsafe impl<C: Comparator> Sync for MemTableMut<C> {}

pub struct MemTable<C: Comparator> {
    memtable: MemTableMut<C>,
}

impl<C: Comparator> From<MemTableMut<C>> for MemTable<C> {
    fn from(memtable: MemTableMut<C>) -> Self {
        MemTable { memtable }
    }
}

#[allow(dead_code)]
impl<C: Comparator> MemTable<C> {
    pub fn log_number(&self) -> u64 {
        self.memtable.log_number()
    }
    pub fn seek_by_internal_key(&self, key: &InternalKey) -> Option<&[u8]> {
        self.memtable.seek_by_internal_key(key)
    }

    pub fn seek_by_key_and_sequence(
        &self,
        key: &Bytes,
        sequence: u64,
    ) -> Result<Option<(ValueTag, &[u8])>> {
        self.memtable.seek_by_key_and_sequence(key, sequence)
    }

    pub fn iter(&self) -> SkipListIterator<InternalKeyComparator<C>> {
        self.memtable.iter()
    }
}
unsafe impl<C: Comparator> Sync for MemTable<C> {}
