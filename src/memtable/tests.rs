use crate::memtable::internal_key::{InternalKey, InternalKeyComparator};
use crate::memtable::table::MemTableMut;
use crate::skip_list::{Comparator, NumberComparator, RandomLevelGenerator};

use crate::format::{ValueTag, ValueType};
use crate::helper::test::*;
use bytes::Bytes;
use std::cmp::Ordering;
use std::sync::Arc;

#[test]
fn test_internal_key_comparator() {
    assert_eq!(
        InternalKeyComparator::<NumberComparator<u32>>::compare(
            InternalKey::new(get_bytes(100), ValueTag::new(1, ValueType::Value).unwrap())
                .as_bytes()
                .as_ref(),
            InternalKey::new(get_bytes(100), ValueTag::new(2, ValueType::Value).unwrap())
                .as_bytes()
                .as_ref(),
        ),
        Ordering::Less
    );

    assert_eq!(
        InternalKeyComparator::<NumberComparator<u32>>::compare(
            InternalKey::new(get_bytes(101), ValueTag::new(1, ValueType::Value).unwrap())
                .as_bytes()
                .as_ref(),
            InternalKey::new(get_bytes(100), ValueTag::new(2, ValueType::Value).unwrap())
                .as_bytes()
                .as_ref(),
        ),
        Ordering::Greater
    );
}

#[test]
fn test_concurrent() {
    let memtable = Arc::new(MemTableMut::<NumberComparator<u32>>::new(
        0,
        Box::new(RandomLevelGenerator::new(19, 0.1)),
        4 * 1024,
    ));

    let data_v1 = generate_data(0, 1000000);

    let pool = threadpool::ThreadPool::new(72);

    for (key, data) in data_v1 {
        let r = memtable.clone();
        let internal_key =
            InternalKey::new(get_bytes(key), ValueTag::new(1, ValueType::Value).unwrap());
        let d = data.clone();
        pool.execute(move || {
            r.add(internal_key, d);
        });
    }

    pool.join();

    let delete_key: Vec<u32> = (90000 as u32..100000 as u32).collect();

    for key in delete_key.iter() {
        let r = memtable.clone();
        let internal_key = InternalKey::new(
            get_bytes(key.clone()),
            ValueTag::new(2, ValueType::TombStone).unwrap(),
        );

        pool.execute(move || {
            r.add(internal_key, Bytes::new());
        });
    }
    pool.join();

    for key in delete_key {
        let r = memtable.clone();

        pool.execute(move || {
            let k = get_bytes(key);
            let result_1 = r.seek_by_key_and_sequence(&k, 1).unwrap();
            match result_1 {
                Some((tag, _)) => assert!(tag.is_value()),
                _ => panic!(format!("Cannot find key: {}", key)),
            };

            let result_2 = r.seek_by_key_and_sequence(&k, 2).unwrap();
            match result_2 {
                Some((tag, _)) => assert!(tag.is_tombstone()),
                _ => panic!(format!("Cannot find key: {}", key)),
            };

            let result_3 = r.seek_by_key_and_sequence(&k, 3).unwrap();
            match result_3 {
                Some((tag, _)) => assert!(tag.is_tombstone()),
                _ => panic!(format!("Cannot find key: {}", key)),
            };
        });
    }

    pool.join();
}
