use crate::assert_as_error;
use crate::error::Error;
use crate::memory_pool::MemoryPool;
use crate::memory_table::variable_number::{decode_usize, encode_usize};
use std::mem::size_of;
use std::ptr::{copy_nonoverlapping, slice_from_raw_parts};

pub type SequenceNumber = u64;
pub type WrappedValueTag = [u8; 8];

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
        let num = match self.ty {
            ValueType::Value => self.sequence_number & 1 << (size_of::<u64>() - 1),
            ValueType::Deletion => self.sequence_number | 1 << (size_of::<u64>() - 1),
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
            ValueType::Deletion
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

#[allow(dead_code)]
pub struct Block {
    pub ptr: *const u8,
    pub internal_key_size: usize,
    pub key: *const u8,
    pub value_size: usize,
    pub value_ptr: *const u8,
    value_tag_ptr: *const u8,
}

impl From<*const u8> for Block {
    fn from(ptr: *const u8) -> Self {
        let (internal_key_size, right) = decode_usize(ptr);
        let value_tag_ptr = right;
        let key = unsafe { right.add(size_of::<WrappedValueTag>()) };
        let (value_size, value) = decode_usize(unsafe { key.clone().add(internal_key_size) });

        Block {
            ptr,
            internal_key_size,
            value_tag_ptr,
            key,
            value_size,
            value_ptr: value,
        }
    }
}

impl Into<*const u8> for Block {
    fn into(self) -> *const u8 {
        self.ptr
    }
}

#[allow(dead_code)]
impl Block {
    pub fn value_tag(&self) -> ValueTag {
        ValueTag::from(self.value_tag_ptr)
    }

    pub fn key_size(&self) -> usize {
        self.internal_key_size - size_of::<ValueTag>()
    }

    pub fn size(&self) -> usize {
        self.value_ptr as usize - self.ptr as usize + self.value_size
    }

    pub fn key_slice_ptr(&self) -> *const [u8] {
        slice_from_raw_parts(self.key, self.key_size())
    }

    pub fn allocate_and_write(
        tag: ValueTag,
        key: &[u8],
        value: &[u8],
        pool: &mut MemoryPool,
    ) -> Self {
        let key_size = key.len();
        let internal_key_size = size_of::<WrappedValueTag>() + key_size;
        let encoded_internal_key_size = encode_usize(internal_key_size);

        let value_size = value.len();
        let encoded_value_size = encode_usize(value_size);

        let size = encoded_internal_key_size.len()
            + internal_key_size
            + encoded_value_size.len()
            + value_size;

        let mut ptr = pool.allocate(size);
        let block_ptr = ptr as *const u8;

        // Write internal_key_size
        unsafe {
            copy_nonoverlapping(
                encoded_internal_key_size.as_ptr(),
                ptr,
                encoded_internal_key_size.len(),
            );
        };
        ptr = unsafe { ptr.add(encoded_internal_key_size.len()) };

        // Write value_tag
        let value_tag_ptr = ptr as *const u8;
        let wrapped_tag: WrappedValueTag = tag.into();
        unsafe {
            copy_nonoverlapping(wrapped_tag.as_ptr(), ptr, size_of::<WrappedValueTag>());
        };
        ptr = unsafe { ptr.add(size_of::<WrappedValueTag>()) };

        // Write key
        let key_ptr = ptr as *const u8;
        unsafe {
            copy_nonoverlapping(key.as_ptr(), ptr, key_size);
        };
        ptr = unsafe { ptr.add(key_size) };

        // Write value_size
        unsafe {
            copy_nonoverlapping(encoded_value_size.as_ptr(), ptr, encoded_value_size.len());
        };
        ptr = unsafe { ptr.add(encoded_value_size.len()) };

        // Write value
        let value_ptr = ptr as *const u8;
        unsafe {
            copy_nonoverlapping(value.as_ptr(), ptr, value_size);
        };

        Block {
            ptr: block_ptr,
            internal_key_size,
            value_tag_ptr,
            key: key_ptr,
            value_size,
            value_ptr,
        }
    }
}
