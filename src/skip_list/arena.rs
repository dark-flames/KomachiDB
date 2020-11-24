use crate::skip_list::node::Node;
use std::mem::align_of;
use std::ptr::null_mut;
use std::sync::RwLock;

struct ArenaCore {
    blocks: Vec<Box<[u8]>>,
    current_block: usize,
    current_block_remaining: usize,
    memory_usage: usize,
    ptr: *mut u8,
}

#[allow(dead_code)]
pub struct Arena {
    core: RwLock<ArenaCore>,
    align: usize,
    block_size: usize,
}

unsafe impl Send for Arena {}

#[allow(dead_code)]
impl Arena {
    pub fn new(block_size: usize) -> Arena {
        Arena {
            core: RwLock::new(ArenaCore {
                blocks: vec![],
                current_block: 0,
                current_block_remaining: 0,
                memory_usage: 0,
                ptr: null_mut(),
            }),
            align: align_of::<Node>(),
            block_size,
        }
    }

    fn create_block(size: usize) -> Box<[u8]> {
        vec![0 as u8; size].into_boxed_slice()
    }

    pub fn memory_usage(&self) -> usize {
        self.core.read().unwrap().memory_usage
    }

    pub fn allocate(&self, mut size: usize) -> *mut u8 {
        let slop = match size % self.align {
            0 => 0,
            others => self.align - others,
        };

        size += slop;

        let mut core = self.core.write().unwrap();

        core.memory_usage += size;

        if size <= core.current_block_remaining {
            let result = core.ptr;
            core.ptr = unsafe { core.ptr.add(size) };
            core.current_block_remaining -= size;

            result
        } else if size > self.block_size / 4 {
            let block = Self::create_block(size);

            core.blocks.push(block);

            core.blocks.last_mut().unwrap().as_mut_ptr()
        } else {
            let block = Self::create_block(self.block_size);

            core.blocks.push(block);

            let result = core.blocks.last_mut().unwrap().as_mut_ptr();

            core.ptr = unsafe { result.clone().add(size) };
            core.current_block = core.blocks.len() - 1;
            core.current_block_remaining = self.block_size - size;

            result
        }
    }
}
