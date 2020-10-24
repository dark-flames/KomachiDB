use bytes::Bytes;
use komachi_db::skip_list::{NumberComparator, RandomLevelGenerator, SkipList};
use rand::seq::SliceRandom;
use rand::{random, thread_rng};
use std::mem::size_of;
use std::ptr::slice_from_raw_parts;
use std::sync::Arc;

fn create_skip_list(max_level: usize) -> SkipList<NumberComparator<u32>> {
    let level_generator = RandomLevelGenerator::new(max_level, 0.5);

    return SkipList::new(1024 * 1024 * 1024 * 3, Box::new(level_generator));
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

pub fn generate_data(size: usize) -> Vec<(u32, Bytes)> {
    let mut data: Vec<u32> = (0 as u32..size as u32).collect();
    let mut rng = thread_rng();
    data.shuffle(&mut rng);
    data.into_iter().map(|k| (k, Bytes::new())).collect()
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
            .map(|(key, _)| get_num(key))
            .collect::<Vec<u32>>()
    );

    let mut visitor = skip_list.visitor();

    for key in set_vec.iter() {
        visitor.seek(&get_bytes(key.clone()));
        assert!(visitor.valid());
    }

    for _ in 0..100 {
        let key = loop {
            let result = random::<u32>();

            if !set_vec.contains(&result) {
                break result;
            }
        };

        visitor.seek(&get_bytes(key));
        assert!(visitor.valid());
    }
}

#[test]
fn test_concurrent() {
    let skip_list = Arc::new(create_skip_list(19));

    // todo: may have some race condition
    let data = generate_data(1000);

    let mut set_vec = data
        .iter()
        .map(|(key, _)| key.clone())
        .collect::<Vec<u32>>();
    set_vec.sort();

    let pool = threadpool::ThreadPool::new(100);

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
            .map(|(key, _)| get_num(key))
            .collect::<Vec<u32>>()
    );

    let mut visitor = skip_list.visitor();

    for key in set_vec.iter() {
        visitor.seek(&get_bytes(key.clone()));
        assert!(visitor.valid());
    }

    for _ in 0..100 {
        let key = loop {
            let result = random::<u32>();

            if !set_vec.contains(&result) {
                break result;
            }
        };

        visitor.seek(&get_bytes(key));
        assert!(visitor.valid());
    }
}
