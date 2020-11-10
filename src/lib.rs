#![feature(slice_ptr_len)]
#![feature(slice_ptr_get)]
#![feature(array_methods)]

#[macro_use]
mod error;
mod core;
mod format;
mod interface;
mod memtable;
mod skip_list;

pub use interface::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
