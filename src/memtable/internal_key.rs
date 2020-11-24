use crate::format::{ValueTag, WrappedValueTag};
use crate::Comparator;
use bytes::{Bytes, BytesMut};
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::mem::size_of;

pub struct InternalKey {
    value_tag: ValueTag,
    key: Bytes,
}

#[allow(dead_code)]
impl InternalKey {
    pub fn new(key: Bytes, value_tag: ValueTag) -> InternalKey {
        InternalKey { key, value_tag }
    }

    pub fn split_key(slice: &[u8]) -> &[u8] {
        Self::split(slice).1
    }

    pub fn split_value_tag(slice: &[u8]) -> ValueTag {
        Self::split(slice).0
    }

    pub fn split(slice: &[u8]) -> (ValueTag, &[u8]) {
        let (tag, key) = slice.split_at(size_of::<WrappedValueTag>());

        let mut wrapped_tag: WrappedValueTag = Default::default();
        wrapped_tag.copy_from_slice(&tag[0..size_of::<WrappedValueTag>()]);

        (wrapped_tag.into(), key)
    }
    pub fn as_bytes(&self) -> Bytes {
        let wrapped_tag: WrappedValueTag = self.value_tag.into();
        let mut result = BytesMut::from(wrapped_tag.to_vec().as_slice());
        result.extend_from_slice(self.key.as_ref());

        result.freeze()
    }
}

impl Into<Bytes> for InternalKey {
    fn into(self) -> Bytes {
        self.as_bytes()
    }
}

impl From<&[u8]> for InternalKey {
    fn from(slice: &[u8]) -> Self {
        let (tag, key) = Self::split(slice);
        InternalKey {
            key: Bytes::copy_from_slice(key),
            value_tag: tag,
        }
    }
}

pub struct InternalKeyComparator<C: Comparator> {
    _marker: PhantomData<C>,
}

impl<C: Comparator> Comparator for InternalKeyComparator<C> {
    fn compare(a: &[u8], b: &[u8]) -> Ordering {
        let (a_tag, a_key) = InternalKey::split(a);
        let (b_tag, b_key) = InternalKey::split(b);

        match C::compare(a_key, b_key) {
            Ordering::Equal => a_tag.sequence_number.cmp(&b_tag.sequence_number),
            others => others,
        }
    }
}
