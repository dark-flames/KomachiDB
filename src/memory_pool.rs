#[allow(dead_code)]
pub struct MemoryPool<'pool> {
    blocks: Vec<Box<[u8]>>,
    block_size: usize,
    remaining_space_ref: &'pool mut [u8],
    memory_usage: usize,
}

#[allow(dead_code)]
impl<'pool> MemoryPool<'pool> {
    pub fn size(&self) -> usize {
        self.memory_usage
    }

    pub fn allocate(&'pool mut self, bytes: usize) -> &'pool mut [u8] {
        self.memory_usage += bytes;

        if bytes < self.remaining_space_ref.len() {
            let (result, new_ref) = self.remaining_space_ref.split_at_mut(bytes + 1);

            self.remaining_space_ref = new_ref;

            result
        } else {
            self.allocate_fallback(bytes)
        }
    }

    fn create_block(bytes: usize) -> Box<[u8]> {
        Vec::<u8>::with_capacity(bytes).into_boxed_slice()
    }

    #[inline(always)]
    fn allocate_new_block(&'pool mut self, bytes: usize) -> &'pool mut [u8] {
        let block_box = Vec::<u8>::with_capacity(bytes).into_boxed_slice();
        self.blocks.push(block_box);

        self.blocks.last_mut().unwrap().as_mut()
    }

    fn allocate_fallback(&'pool mut self, bytes: usize) -> &'pool mut [u8] {
        if bytes > self.block_size / 4 {
            let block_box = Self::create_block(bytes);
            self.blocks.push(block_box);

            self.blocks.last_mut().unwrap().as_mut()
        } else {
            // allocate a new block and waste remaining space
            let block_box = Self::create_block(self.block_size);
            self.blocks.push(block_box);

            let (result, new_ref) = self
                .blocks
                .last_mut()
                .unwrap()
                .as_mut()
                .split_at_mut(bytes + 1);

            self.remaining_space_ref = new_ref;

            result
        }
    }
}
