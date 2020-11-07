use super::{NumberComparator, RandomLevelGenerator, SkipList};
use crate::Data;
use rand::seq::SliceRandom;
use rand::{random, thread_rng};
use std::sync::Arc;

fn create_skip_list(max_level: usize) -> SkipList<NumberComparator<u32>> {
    let level_generator = RandomLevelGenerator::new(max_level, 0.5);

    SkipList::new(Box::new(level_generator), 4096)
}

pub fn generate_data(size: usize) -> Vec<(u32, u32)> {
    let mut data: Vec<u32> = (0 as u32..size as u32).collect();
    let mut rng = thread_rng();
    data.shuffle(&mut rng);
    data.into_iter().map(|k| (k, k)).collect()
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
        skip_list.insert(key, data);
    }

    assert_eq!(
        set_vec,
        skip_list
            .iter()
            .map(|(key, _)| u32::from_ref(key).clone())
            .collect::<Vec<u32>>()
    );

    let mut visitor = skip_list.visitor();

    for key in set_vec.iter() {
        visitor.seek(key);
        assert!(visitor.valid());
    }

    for _ in 0..100 {
        let key = loop {
            let result = random::<u32>();

            if !set_vec.contains(&result) {
                break result;
            }
        };

        visitor.seek(&key);
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
            r.insert(key, data);
        });
    }

    pool.join();

    assert_eq!(
        set_vec,
        skip_list
            .iter()
            .map(|(key, _)| u32::from_ref(key).clone())
            .collect::<Vec<u32>>()
    );

    let mut visitor = skip_list.visitor();

    for key in set_vec.iter() {
        visitor.seek(key);
        assert!(visitor.valid());
    }
}
