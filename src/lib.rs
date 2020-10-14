#![feature(slice_ptr_len)]
#[macro_use]
mod error;
mod format;
mod implement;
mod interface;
mod memory_pool;
mod memory_table;
mod skip_list;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
