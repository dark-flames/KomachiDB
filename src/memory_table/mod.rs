use crate::comparator::Comparator;
use crate::error::Error;
use crate::memory_pool::MemoryPool;
use crate::memory_table::format::{Block, ValueTag};
use crate::skip_list::SkipList;
use format::{SequenceNumber, ValueType};

mod format;
mod variable_number;

#[allow(dead_code)]
pub struct MemTable<C: Comparator> {
    pool: MemoryPool,
    skip_list: SkipList<C>,
}

#[allow(dead_code)]
impl<C: Comparator> MemTable<C> {
    pub fn add(
        &mut self,
        sequence_number: SequenceNumber,
        value_type: ValueType,
        key: &[u8],
        value: &[u8],
    ) -> Result<(), Error> {
        let value_tag = ValueTag::new(sequence_number, value_type)?;

        let block = Block::allocate_and_write(value_tag, key, value, &mut self.pool);

        self.skip_list.insert(block.key_slice_ptr(), block.ptr);

        Ok(())
    }
}
