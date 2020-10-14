use crate::assert_as_error;
use crate::error::Error;
use crate::interface::Key;
use std::mem::size_of;

pub type SequenceNumber = u64;
pub type WrappedValueTag = u64;

pub enum ValueType {
    Value = 0,
    Deletion = 1,
}

pub struct ValueTag {
    sequence_number: SequenceNumber,
    ty: ValueType,
}

#[allow(dead_code)]
impl ValueTag {
    pub fn new(sequence_number: SequenceNumber, ty: ValueType) -> Result<ValueTag, Error> {
        assert_as_error!(
            sequence_number < (1 << (size_of::<u64>() * 8)),
            Error::SequenceNumberOverflow
        );

        Ok(ValueTag {
            sequence_number,
            ty,
        })
    }
}

impl Into<WrappedValueTag> for ValueTag {
    fn into(self) -> WrappedValueTag {
        match self.ty {
            ValueType::Value => self.sequence_number & 1 << (size_of::<u64>() - 1),
            ValueType::Deletion => self.sequence_number | 1 << (size_of::<u64>() - 1),
        }
    }
}

impl From<WrappedValueTag> for ValueTag {
    fn from(wrapped: WrappedValueTag) -> Self {
        let sequence_number = wrapped & 1 << (size_of::<u64>() * 8 - 1);
        let ty = if (wrapped & 1 << (size_of::<u64>() * 8 - 1)) == 0 {
            ValueType::Value
        } else {
            ValueType::Deletion
        };

        ValueTag {
            sequence_number,
            ty,
        }
    }
}

#[allow(dead_code)]
pub struct InternalKey<K: Key> {
    value_tag: WrappedValueTag,
    key: K,
}
