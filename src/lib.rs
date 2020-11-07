#![feature(slice_ptr_len)]
#![feature(slice_ptr_get)]
#![feature(array_methods)]

#[macro_use]
mod error;
mod format;
mod memtable;
pub mod skip_list;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
