use crate::assert_as_error;
use crate::error::Error;
use std::mem::size_of;

pub type SequenceNumber = u64;

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

impl Into<u64> for ValueTag {
    fn into(self) -> u64 {
        match self.ty {
            ValueType::Value => self.sequence_number & 1 << (size_of::<u64>() - 1),
            ValueType::Deletion => self.sequence_number | 1 << (size_of::<u64>() - 1),
        }
    }
}

impl From<u64> for ValueTag {
    fn from(wrapped: u64) -> Self {
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
