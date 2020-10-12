#![feature(slice_ptr_len)]
mod implement;
mod interface;
mod memory_pool;
mod skip_list;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
