use super::internal_key::{InternalKey, InternalKeyComparator};
use crate::format::ValueTag;
use crate::skip_list::{Comparator, SkipList};
use bytes::Bytes;

#[allow(dead_code)]
struct MemTable<C: Comparator> {
    skip_list: SkipList<InternalKeyComparator<C>>,
}

unsafe impl<C: Comparator> Sync for MemTable<C> {}

#[allow(dead_code)]
impl<C: Comparator> MemTable<C> {
    pub fn insert(&mut self, key: impl Into<Bytes>, value: impl Into<Bytes>, value_tag: ValueTag) {
        let internal_key = InternalKey::new(key, value_tag);
        self.skip_list.insert(internal_key.into(), value.into())
    }
}
