use crate::skip_list::arena::Arena;
use crate::skip_list::MAX_HEIGHT;
use bytes::Bytes;
use std::intrinsics::write_bytes;
use std::mem::size_of;
use std::ptr::write;
use std::sync::atomic::{AtomicU32, Ordering};

#[allow(dead_code)]
#[derive(Debug)]
#[repr(C)]
pub struct Node {
    key: Bytes,
    value: Bytes,
    height: usize,
    next: [AtomicU32; MAX_HEIGHT],
}

#[allow(dead_code)]
impl Node {
    pub fn allocate_with_arena(
        key: Bytes,
        value: Bytes,
        height: usize,
        arena: &mut Arena<Node>,
    ) -> u32 {
        let size = size_of::<Self>() - (MAX_HEIGHT - 1 - height) * size_of::<u32>();

        unsafe {
            let offset = arena.allocate(size as u32);
            let ptr = arena.get_mut(offset);
            let node = ptr.as_mut().unwrap();

            write(&mut node.key, key);
            write(&mut node.value, value);
            write(&mut node.height, height);
            write_bytes(node.next.as_mut_ptr(), 0, height + 1);

            offset
        }
    }

    pub fn next_offset(&self, level: usize) -> u32 {
        if level <= self.height {
            self.next[level].load(Ordering::SeqCst)
        } else {
            0
        }
    }

    pub fn set_next(&mut self, level: usize, offset: u32) {
        assert!(level <= self.height);

        self.next[level].store(offset, Ordering::SeqCst)
    }

    pub fn is_head(&self) -> bool {
        self.key.is_empty() && self.value.is_empty()
    }

    pub fn key(&self) -> &Bytes {
        &self.key
    }

    pub fn value(&self) -> &Bytes {
        &self.value
    }

    pub fn height(&self) -> usize {
        self.height
    }
}
