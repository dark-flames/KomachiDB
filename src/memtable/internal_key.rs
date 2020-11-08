use crate::format::{ValueTag, WrappedValueTag};
use crate::skip_list::Comparator;
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
        slice.split_at(size_of::<WrappedValueTag>()).1
    }

    pub fn split_value_tag(slice: &[u8]) -> ValueTag {
        let mut wrapped_tag: WrappedValueTag = Default::default();
        wrapped_tag.copy_from_slice(&slice[0..4]);
        wrapped_tag.into()
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
        InternalKey {
            key: Bytes::copy_from_slice(Self::split_key(slice)),
            value_tag: Self::split_value_tag(slice),
        }
    }
}

pub struct InternalKeyComparator<C: Comparator> {
    _marker: PhantomData<C>,
}

impl<C: Comparator> Comparator for InternalKeyComparator<C> {
    fn compare(a: &[u8], b: &[u8]) -> Ordering {
        match C::compare(InternalKey::split_key(a), InternalKey::split_key(b)) {
            Ordering::Equal => {
                let a_sequence = InternalKey::split_value_tag(a).sequence_number;
                let b_sequence = InternalKey::split_value_tag(b).sequence_number;
                a_sequence.cmp(&b_sequence)
            }
            others => others,
        }
    }
}
