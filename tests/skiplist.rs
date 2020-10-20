use bytes::Bytes;
use komachi_db::skip_list::{NumberComparator, RandomLevelGenerator, SkipList};
use rand::random;
use std::collections::HashSet;
use std::mem::size_of;
use std::ptr::slice_from_raw_parts;

fn create_skip_list(max_level: usize) -> SkipList<NumberComparator<u32>> {
    let level_generator = RandomLevelGenerator::new(max_level, 0.5);

    return SkipList::new(1024 * 300, Box::new(level_generator));
}

pub fn get_bytes(n: u32) -> Bytes {
    let ptr = Box::into_raw(Box::new(n)) as *const u8;
    Bytes::copy_from_slice(unsafe {
        slice_from_raw_parts(ptr, size_of::<u32>())
            .as_ref()
            .unwrap()
    })
}

pub fn get_num(bytes: &Bytes) -> u32 {
    unsafe { *(bytes.as_ref().as_ptr() as *const u32) }
}

#[test]
fn test_simple() {
    let mut skip_list = create_skip_list(3);

    skip_list.insert(get_bytes(3), get_bytes(3));
    skip_list.insert(get_bytes(5), get_bytes(5));
    skip_list.insert(get_bytes(6), get_bytes(6));
    skip_list.insert(get_bytes(1), get_bytes(1));

    assert_eq!(
        vec![1, 3, 5, 6],
        skip_list
            .iter()
            .map(|(key, _)| { get_num(key) })
            .collect::<Vec<u32>>()
    );
}

#[test]
fn random_test_insert() {
    let mut skip_list = create_skip_list(9);

    let mut set = HashSet::new();

    for _ in 0..100 {
        let key = loop {
            let result = random::<u32>();

            if !set.contains(&result) {
                break result;
            }
        };

        skip_list.insert(get_bytes(key), get_bytes(key));
        set.insert(key);
    }

    let mut set_vec = set.iter().map(|key| key.clone()).collect::<Vec<u32>>();
    set_vec.sort();
    assert_eq!(
        set_vec,
        skip_list
            .iter()
            .map(|(key, _)| get_num(key))
            .collect::<Vec<u32>>()
    );

    let mut visitor = skip_list.visitor();

    for key in set_vec {
        visitor.seek(&get_bytes(key));
        assert!(visitor.valid());
    }

    for _ in 0..100 {
        let key = loop {
            let result = random::<u32>();

            if !set.contains(&result) {
                break result;
            }
        };

        visitor.seek(&get_bytes(key));
        assert!(visitor.valid());
    }
}
