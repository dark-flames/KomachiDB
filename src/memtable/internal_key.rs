use crate::format::{ValueTag, WrappedValueTag};
use crate::skip_list::Comparator;
use bytes::{Bytes, BytesMut};
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::mem::size_of;

pub struct InternalKey(Bytes);

#[allow(dead_code)]
impl InternalKey {
    pub fn new(key: impl Into<Bytes>, value_tag: ValueTag) -> Self {
        let wrapped_tag: WrappedValueTag = value_tag.into();
        let mut internal_key = BytesMut::from(wrapped_tag.as_slice());
        internal_key.extend_from_slice(key.into().as_ref());

        InternalKey(internal_key.freeze())
    }

    pub fn split_key(slice: &[u8]) -> &[u8] {
        slice.split_at(size_of::<WrappedValueTag>()).1
    }

    pub fn get_value_tag(slice: &[u8]) -> ValueTag {
        let mut wrapped_tag: WrappedValueTag = Default::default();
        wrapped_tag.copy_from_slice(&slice[0..4]);
        wrapped_tag.into()
    }
}

impl Into<Bytes> for InternalKey {
    fn into(self) -> Bytes {
        self.0
    }
}

pub struct InternalKeyComparator<C: Comparator> {
    _marker: PhantomData<C>,
}

impl<C: Comparator> Comparator for InternalKeyComparator<C> {
    fn compare(a: &[u8], b: &[u8]) -> Ordering {
        match C::compare(InternalKey::split_key(a), InternalKey::split_key(b)) {
            Ordering::Equal => {
                let a_sequence = InternalKey::get_value_tag(a).sequence_number;
                let b_sequence = InternalKey::get_value_tag(b).sequence_number;
                a_sequence.cmp(&b_sequence)
            }
            others => others,
        }
    }
}
