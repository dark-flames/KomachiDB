use crate::skip_list::arena::Arena;
use crate::skip_list::MAX_HEIGHT;
use bytes::Bytes;
use std::intrinsics::copy_nonoverlapping;
use std::mem::size_of;
use std::ptr::write_bytes;
use std::ptr::{null, null_mut, slice_from_raw_parts, write};
use std::sync::atomic::{AtomicPtr, Ordering};

#[allow(dead_code)]
#[derive(Debug)]
#[repr(C)]
pub struct Node {
    key: *const u8,
    value: *const u8,
    size: usize,
    height: usize,
    next: [AtomicPtr<Node>; MAX_HEIGHT],
}

#[allow(dead_code)]
impl Node {
    pub fn allocate_with_arena(
        key: Bytes,
        value: Bytes,
        height: usize,
        arena: &Arena,
    ) -> *mut Node {
        let node_size = size_of::<Self>() - (MAX_HEIGHT - 1 - height) * size_of::<u32>();
        let key_size = key.len();
        let value_size = value.len();
        let data_size = key_size + value_size;

        let mut head = arena.allocate(node_size + data_size);

        // write node
        unsafe {
            let node = (head as *mut Node).as_mut().unwrap();
            head = head.add(node_size);
            let key_ptr = head;
            copy_nonoverlapping(key.as_ptr(), key_ptr, key_size);
            let value_ptr = head.add(key_size);
            copy_nonoverlapping(value.as_ptr(), value_ptr, value_size);
            write(&mut node.key, key_ptr);
            write(&mut node.value, value_ptr);
            write(&mut node.size, data_size);
            write(&mut node.height, height);
            write_bytes(node.next.as_mut_ptr(), 0, height + 1);

            node
        }
    }

    pub fn head(height: usize, arena: &Arena) -> *mut Node {
        let node_size =
            size_of::<Self>() - (MAX_HEIGHT - 1 - height) * size_of::<AtomicPtr<Node>>();

        let head = arena.allocate(node_size);

        // write node
        unsafe {
            let node = (head as *mut Node).as_mut().unwrap();

            write(&mut node.key, null());
            write(&mut node.value, null());
            write(&mut node.size, 0);
            write(&mut node.height, height);
            write_bytes(node.next.as_mut_ptr(), 0, height + 1);

            node
        }
    }

    pub fn next(&self, level: usize) -> *mut Node {
        if level <= self.height {
            self.next[level].load(Ordering::SeqCst)
        } else {
            null_mut()
        }
    }

    pub fn set_next(&mut self, level: usize, node: *mut Node) {
        assert!(level <= self.height);

        self.next[level].store(node, Ordering::SeqCst)
    }

    pub fn get_next_atomic(&mut self, level: usize) -> &AtomicPtr<Node> {
        assert!(level <= self.height);

        &self.next[level]
    }

    pub fn is_head(&self) -> bool {
        self.key.is_null()
    }

    fn key_size(&self) -> usize {
        (self.value as usize) - (self.key as usize)
    }

    fn value_size(&self) -> usize {
        self.size - self.key_size()
    }

    pub fn key(&self) -> Option<&[u8]> {
        if !self.is_head() {
            Some(unsafe {
                slice_from_raw_parts(self.key, self.key_size())
                    .as_ref()
                    .unwrap()
            })
        } else {
            None
        }
    }

    pub fn value(&self) -> Option<&[u8]> {
        if !self.is_head() {
            Some(unsafe {
                slice_from_raw_parts(self.value, self.value_size())
                    .as_ref()
                    .unwrap()
            })
        } else {
            None
        }
    }

    pub fn height(&self) -> usize {
        self.height
    }
}
