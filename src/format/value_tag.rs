use crate::assert_as_error;
use crate::error::{Error, Result};
use std::mem::size_of;

pub type SequenceNumber = u64;
pub type WrappedValueTag = [u8; 8];

#[allow(dead_code)]
pub enum ValueType {
    Value = 0,
    TombStone = 1,
}

#[allow(dead_code)]
pub struct ValueTag {
    pub sequence_number: SequenceNumber,
    pub ty: ValueType,
}

#[allow(dead_code)]
impl ValueTag {
    pub fn new(sequence_number: SequenceNumber, ty: ValueType) -> Result<ValueTag> {
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
        let num = match self.ty {
            ValueType::Value => self.sequence_number & 1 << (size_of::<u64>() - 1),
            ValueType::TombStone => self.sequence_number | 1 << (size_of::<u64>() - 1),
        };

        num.to_be_bytes()
    }
}

impl From<WrappedValueTag> for ValueTag {
    fn from(wrapped_tag: WrappedValueTag) -> Self {
        let num = u64::from_be_bytes(wrapped_tag);
        let sequence_number = num & 1 << (size_of::<u64>() * 8 - 1);
        let ty = if (num & 1 << (size_of::<u64>() * 8 - 1)) == 0 {
            ValueType::Value
        } else {
            ValueType::TombStone
        };

        ValueTag {
            sequence_number,
            ty,
        }
    }
}

impl From<*const u8> for ValueTag {
    fn from(ptr: *const u8) -> Self {
        let slice = unsafe { *(ptr as *const u64) }.to_be_bytes();
        Self::from(slice)
    }
}
