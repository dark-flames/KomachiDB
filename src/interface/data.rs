use std::mem::size_of;
use std::ptr::{slice_from_raw_parts, write};

pub unsafe trait Data {
    fn as_ref(&self) -> &[u8];

    fn write_to(self, ptr: *mut u8);

    fn from_ref(bytes: &[u8]) -> &Self;

    fn size(&self) -> usize;
}

unsafe impl<T: 'static + Sized + Ord + Copy + Sync> Data for T {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            slice_from_raw_parts((self as *const T) as *const u8, size_of::<Self>())
                .as_ref()
                .unwrap()
        }
    }

    fn write_to(self, ptr: *mut u8) {
        unsafe { write(ptr as *mut T, self) };
    }

    fn from_ref(bytes: &[u8]) -> &Self {
        unsafe { (bytes.as_ptr() as *const Self).as_ref().unwrap() }
    }

    fn size(&self) -> usize {
        size_of::<Self>()
    }
}
