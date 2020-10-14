#[allow(dead_code)]
pub struct MemoryPool {
    blocks: Vec<Box<[u8]>>,
    block_size: usize,
    remaining_space_ptr: Option<*mut u8>,
    remaining_space: usize,
    memory_usage: usize,
}

#[allow(dead_code)]
impl MemoryPool {
    pub fn new(block_size: usize) -> MemoryPool {
        MemoryPool {
            blocks: vec![],
            block_size,
            remaining_space_ptr: None,
            remaining_space: 0,
            memory_usage: 0,
        }
    }
    pub fn size(&self) -> usize {
        self.memory_usage
    }

    pub fn allocate(&mut self, bytes: usize) -> *mut u8 {
        self.memory_usage += bytes;

        if bytes < self.remaining_space() {
            let result = self.remaining_space_ptr.unwrap();
            self.remaining_space_ptr =
                Some(unsafe { self.remaining_space_ptr.unwrap().add(bytes) });

            self.remaining_space -= bytes;

            result
        } else {
            self.allocate_fallback(bytes)
        }
    }

    fn remaining_space(&self) -> usize {
        self.remaining_space
    }

    fn create_block(bytes: usize) -> Box<[u8]> {
        vec![0 as u8; bytes].into_boxed_slice()
    }

    fn allocate_fallback(&mut self, bytes: usize) -> *mut u8 {
        if bytes > self.block_size / 4 {
            let block_box = Self::create_block(bytes);

            self.blocks.push(block_box);

            self.blocks.last_mut().unwrap().as_mut_ptr()
        } else {
            // allocate a new block and waste remaining space
            let block_box = Self::create_block(self.block_size);
            self.blocks.push(block_box);

            let result = self.blocks.last_mut().unwrap().as_mut_ptr();
            let mut remaining_space_ptr = result;
            remaining_space_ptr = unsafe { remaining_space_ptr.add(bytes) };

            self.remaining_space_ptr = Some(remaining_space_ptr);
            self.remaining_space = self.block_size - bytes;

            result
        }
    }
}

#[cfg(test)]
mod test {
    use crate::memory_pool::MemoryPool;

    fn create_test_pool() -> MemoryPool {
        MemoryPool::new(4096)
    }

    #[test]
    fn test_big_fallback() {
        let mut pool = create_test_pool();

        pool.allocate(1024);
        let left = pool.remaining_space();
        pool.allocate(1029 * 8); // allocate a new block

        assert_eq!(left, pool.remaining_space());
    }

    #[test]
    fn test_small_fallback() {
        let mut pool = create_test_pool();

        pool.allocate(1024 * 3 + 512);
        pool.allocate(640); // waste left space

        assert_eq!(pool.remaining_space, 4096 - 640);
    }
}
