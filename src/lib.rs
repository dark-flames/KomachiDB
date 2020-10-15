#![feature(slice_ptr_len)]
#![feature(slice_ptr_get)]

#[macro_use]
mod error;
mod comparator;
mod implement;
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
