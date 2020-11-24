use super::{NumberComparator, RandomLevelGenerator, SkipList};
use crate::helper::test::*;
use rand::random;
use std::sync::Arc;

fn create_skip_list(max_level: usize) -> SkipList<NumberComparator<u32>> {
    let level_generator = RandomLevelGenerator::new(max_level, 0.1);

    SkipList::new(Box::new(level_generator), 4096)
}

#[test]
fn random_test_insert() {
    let skip_list = create_skip_list(9);

    let data = generate_data(0, 100);
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

    let data = generate_data(0, 1000000);

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
