use super::{NumberComparator, RandomLevelGenerator, SkipList};
use bytes::Bytes;
use rand::seq::SliceRandom;
use rand::{random, thread_rng};
use std::mem::size_of;
use std::ptr::slice_from_raw_parts;
use std::sync::Arc;

fn create_skip_list(max_level: usize) -> SkipList<NumberComparator<u32>> {
    let level_generator = RandomLevelGenerator::new(max_level, 0.5);

    SkipList::new(Box::new(level_generator), 4096)
}

pub fn generate_data(size: usize) -> Vec<(u32, Bytes)> {
    let mut data: Vec<u32> = (0 as u32..size as u32).collect();
    let mut rng = thread_rng();
    data.shuffle(&mut rng);
    data.into_iter().map(|k| (k, get_bytes(k))).collect()
}

pub fn get_u32(bytes: &[u8]) -> u32 {
    unsafe { *(bytes.as_ptr() as *const u32) }
}

pub fn get_bytes(n: u32) -> Bytes {
    let ptr = Box::into_raw(Box::new(n)) as *const u8;
    Bytes::copy_from_slice(unsafe {
        slice_from_raw_parts(ptr, size_of::<u32>())
            .as_ref()
            .unwrap()
    })
}

#[test]
fn random_test_insert() {
    let skip_list = create_skip_list(9);

    let data = generate_data(100);
    let mut set_vec = data
        .iter()
        .map(|(key, _)| key.clone())
        .collect::<Vec<u32>>();
    set_vec.sort();

    for (key, data) in data {
        skip_list.insert(get_bytes(key), data);
    }

    assert_eq!(
        set_vec,
        skip_list
            .iter()
            .map(|(key, _)| get_u32(key).clone())
            .collect::<Vec<u32>>()
    );

    for key in set_vec.iter() {
        let mut visitor = skip_list.visitor();
        visitor.seek(get_bytes(key.clone()).as_ref());
        assert!(visitor.valid());
    }

    for _ in 0..100 {
        let key = loop {
            let result = random::<u32>();

            if !set_vec.contains(&result) {
                break result;
            }
        };
        let mut visitor = skip_list.visitor();
        visitor.seek(get_bytes(key.clone()).as_ref());
        assert!(!visitor.valid());
    }
}

#[test]
fn test_concurrent() {
    let skip_list = Arc::new(create_skip_list(19));

    let data = generate_data(1000000);

    let mut set_vec = data
        .iter()
        .map(|(key, _)| key.clone())
        .collect::<Vec<u32>>();
    set_vec.sort();

    let pool = threadpool::ThreadPool::new(72);

    for (key, data) in data.clone() {
        let r = skip_list.clone();
        pool.execute(move || {
            r.insert(get_bytes(key), data);
        });
    }

    pool.join();

    assert_eq!(
        set_vec,
        skip_list
            .iter()
            .map(|(key, _)| get_u32(key).clone())
            .collect::<Vec<u32>>()
    );

    for key in set_vec.iter() {
        let mut visitor = skip_list.visitor();
        visitor.seek(get_bytes(key.clone()).as_ref());
        assert!(visitor.valid());
    }
}
