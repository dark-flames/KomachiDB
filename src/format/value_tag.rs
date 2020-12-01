use crate::assert_as_error;
use crate::error::{Error, Result};
use std::mem::size_of;

pub type SequenceNumber = u64;
pub type WrappedValueTag = [u8; 8];

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ValueType {
    Value = 0,
    TombStone = 1,
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct ValueTag {
    pub sequence_number: SequenceNumber,
    pub ty: ValueType,
}

#[allow(dead_code)]
impl ValueTag {
    pub fn new(sequence_number: SequenceNumber, ty: ValueType) -> Result<ValueTag> {
        assert_as_error!(sequence_number < u64::MAX, Error::SequenceNumberOverflow);

        Ok(ValueTag {
            sequence_number,
            ty,
        })
    }

    pub fn is_value(&self) -> bool {
        self.ty == ValueType::Value
    }

    pub fn is_tombstone(&self) -> bool {
        self.ty == ValueType::TombStone
    }
}

impl Into<WrappedValueTag> for ValueTag {
    fn into(self) -> WrappedValueTag {
        let num: u64 = match self.ty {
            ValueType::Value => self.sequence_number & !(1u64 << (size_of::<u64>() * 8 - 1)),
            ValueType::TombStone => self.sequence_number | 1u64 << (size_of::<u64>() * 8 - 1),
        };

        num.to_ne_bytes()
    }
}

impl From<WrappedValueTag> for ValueTag {
    fn from(wrapped_tag: WrappedValueTag) -> Self {
        let num = u64::from_ne_bytes(wrapped_tag);
        let sequence_number = num & !(1u64 << (size_of::<u64>() * 8 - 1));
        let ty = if (num & 1u64 << (size_of::<u64>() * 8 - 1)) == 0 {
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
        let slice = unsafe { *(ptr as *const u64) }.to_ne_bytes();
        Self::from(slice)
    }
}

#[test]
fn test_value() {
    use rand::random;
    let sequence = random::<u32>() as u64;
    let raw = ValueTag::new(sequence, ValueType::Value).unwrap();
    let wrapped: WrappedValueTag = raw.into();
    let result: ValueTag = wrapped.into();
    assert_eq!(result.sequence_number, sequence);
    assert_eq!(result.ty, ValueType::Value);
}

#[test]
fn test_tombstone() {
    use rand::random;
    let sequence = random::<u32>() as u64;
    let raw = ValueTag::new(sequence, ValueType::TombStone).unwrap();
    let wrapped: WrappedValueTag = raw.into();
    let result: ValueTag = wrapped.into();
    assert_eq!(result.sequence_number, sequence);
    assert_eq!(result.ty, ValueType::TombStone);
}
