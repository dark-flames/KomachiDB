use crate::skip_list::node::Node;
use std::mem::align_of;
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicU32, Ordering};

#[allow(dead_code)]
pub struct Arena {
    vec: Vec<u8>,
    ptr: *mut u8,
    internal_offset: AtomicU32,
    align: u32,
}

impl Arena {
    pub fn with_capacity(size: u32) -> Arena {
        let mut vec = Vec::with_capacity(size as usize);
        let ptr = vec.as_mut_ptr();
        Arena {
            vec,
            ptr,
            internal_offset: AtomicU32::new(0),
            align: align_of::<Node>() as u32,
        }
    }

    pub fn capacity(&self) -> usize {
        self.vec.capacity()
    }

    pub fn allocate(&self, size: u32) -> u32 {
        let size_mod = size & (self.align - 1);
        let slop = if size_mod == 0 {
            0
        } else {
            self.align - size_mod
        };

        let need = size + slop;

        let offset = self.internal_offset.fetch_add(need, Ordering::SeqCst);

        assert!(offset + need <= self.capacity() as u32);

        offset + 1
    }

    pub unsafe fn get_mut_node(&self, offset: u32) -> *mut Node {
        if offset == 0 {
            null_mut()
        } else {
            let internal_offset = offset - 1;
            self.ptr.clone().add(internal_offset as usize) as *mut Node
        }
    }

    pub unsafe fn get_node(&self, offset: u32) -> *const Node {
        if offset == 0 {
            null()
        } else {
            let internal_offset = offset - 1;
            self.ptr.clone().add(internal_offset as usize) as *const Node
        }
    }

    pub fn get_offset(&self, ptr: *const Node) -> u32 {
        let head_addr = self.ptr as usize;
        let addr = ptr as usize;

        if (head_addr <= addr) && (addr < head_addr + self.capacity()) {
            (addr - head_addr) as u32 + 1
        } else {
            0
        }
    }
}

unsafe impl Send for Arena {}
