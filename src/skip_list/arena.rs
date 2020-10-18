use std::marker::PhantomData;
use std::mem::align_of;
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicU32, Ordering};

#[allow(dead_code)]
pub struct Arena<T: Sized> {
    vec: Vec<u8>,
    ptr: *mut u8,
    internal_offset: AtomicU32,
    align: u32,
    _marker: PhantomData<T>,
}

impl<T> Arena<T> {
    pub fn with_capacity(size: u32) -> Arena<T> {
        let mut vec = Vec::with_capacity(size as usize);
        let ptr = vec.as_mut_ptr();
        Arena {
            vec,
            ptr,
            internal_offset: AtomicU32::new(0),
            align: align_of::<T>() as u32,
            _marker: Default::default(),
        }
    }

    pub fn capacity(&self) -> usize {
        self.vec.capacity()
    }

    pub fn allocate(&mut self, size: u32) -> u32 {
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

    pub unsafe fn get_mut(&self, offset: u32) -> *mut T {
        if offset == 0 {
            null_mut()
        } else {
            let internal_offset = offset - 1;
            self.ptr.clone().add(internal_offset as usize) as *mut T
        }
    }

    pub unsafe fn get(&self, offset: u32) -> *const T {
        if offset == 0 {
            null()
        } else {
            let internal_offset = offset - 1;
            self.ptr.clone().add(internal_offset as usize) as *const T
        }
    }

    pub fn get_offset(&self, ptr: *const T) -> u32 {
        let head_addr = self.ptr as usize;
        let addr = ptr as usize;

        if (head_addr <= addr) && (addr < head_addr + self.capacity()) {
            (addr - head_addr) as u32 + 1
        } else {
            0
        }
    }
}
