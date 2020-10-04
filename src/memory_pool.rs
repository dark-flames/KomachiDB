#[allow(dead_code)]
pub struct MemoryPool {
    blocks: Vec<Box<[u8]>>,
    block_size: usize,
    size_remaining: usize,
    current_ptr: *mut u8,
    memory_usage: usize,
}

#[allow(dead_code)]
impl MemoryPool {
    pub fn size(&self) -> usize {
        self.memory_usage
    }

    pub fn allocate(&mut self, bytes: usize) -> *mut u8 {
        self.memory_usage += bytes;

        if bytes < self.size_remaining {
            let result = self.current_ptr;

            self.size_remaining -= bytes;
            self.current_ptr = unsafe { self.current_ptr.add(bytes) };

            result
        } else {
            self.allocate_fallback(bytes)
        }
    }

    fn allocate_new_block(&mut self, bytes: usize) -> *mut u8 {
        let mut block_box = Vec::<u8>::with_capacity(bytes).into_boxed_slice();
        let pointer = block_box.as_mut_ptr();
        self.blocks.push(block_box);

        pointer
    }

    fn allocate_fallback(&mut self, bytes: usize) -> *mut u8 {
        if bytes > self.block_size / 4 {
            self.allocate_new_block(bytes)
        } else {
            // allocate a new block and waste remaining space
            let result = self.allocate_new_block(self.block_size);
            self.size_remaining = self.block_size - bytes;
            self.current_ptr = unsafe { result.clone().add(bytes) };

            result
        }
    }
}
