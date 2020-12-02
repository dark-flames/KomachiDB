use crate::error::Result;
use crate::logger::chunk::{Chunk, ChunkType};
use crate::logger::record::{Record, RecordChunk};
use crate::logger::LogManager;
use bytes::{Bytes, BytesMut};
use rand::distributions::Alphanumeric;
use rand::{random, Rng};
use std::convert::TryInto;
use std::env::temp_dir;
use std::fs::create_dir;

fn create_random_bytes(size: usize) -> Bytes {
    let rng = &mut rand::thread_rng();

    Bytes::from(
        rng.sample_iter(&Alphanumeric)
            .take(size)
            .collect::<String>(),
    )
}

#[test]
fn test_chunk_encode() {
    let data = create_random_bytes(10);
    let chunk = Chunk::new(vec![data.as_ref()], ChunkType::Full);
    assert!(chunk.check_crc32());
    let slices: Vec<&[u8]> = chunk.as_ref().into();

    let slice = slices
        .into_iter()
        .fold(BytesMut::new(), |mut carry, item| {
            carry.extend_from_slice(item);

            carry
        })
        .freeze();

    let chunk_decode: Chunk = slice.as_ref().into();

    assert_eq!(chunk.ty(), chunk_decode.ty());
    assert_eq!(chunk.crc32(), chunk_decode.crc32());
    assert_eq!(chunk.len(), chunk_decode.len());
    assert_eq!(chunk.data_len(), chunk_decode.data_len());
}

#[test]
fn test_record_encode() {
    let key = create_random_bytes(10);
    let value = create_random_bytes(10);

    let record = Record::new(key.as_ref(), value.as_ref());
    let chunks = record.get_chunks(4 * 1024, 4 * 1024).0;
    let first_chunk = match chunks.first().unwrap() {
        RecordChunk::Normal(c) => Some(c),
        _ => None,
    }
    .unwrap();

    let slices = first_chunk.data.clone();

    let slice = slices
        .into_iter()
        .fold(BytesMut::new(), |mut carry, item| {
            carry.extend_from_slice(item);

            carry
        })
        .freeze();

    let decode_record: Record = slice.as_ref().into();

    assert_eq!(chunks.len(), 1);
    assert_eq!(first_chunk.ty(), ChunkType::Full);
    assert_eq!(decode_record.value_size(), record.value_size());
    assert_eq!(decode_record.key_size(), record.key_size());
    assert_eq!(decode_record.len(), record.len());
}

#[test]
fn test_large_record_encode() {
    let key = create_random_bytes(10);
    let value = create_random_bytes(100000);

    let record = Record::new(key.as_ref(), value.as_ref());
    let chunks = record.get_chunks(1024, 4 * 1024).0;
    let first_chunk = match chunks.first().unwrap() {
        RecordChunk::Normal(c) => Some(c),
        _ => None,
    }
    .unwrap();
    let mut iter = chunks.iter().rev();
    let last_chunk = loop {
        match iter.next() {
            Some(RecordChunk::Normal(c)) => {
                break Some(c);
            }
            None => break None,
            _ => (),
        };
    }
    .unwrap();

    assert_eq!(first_chunk.len(), 1024);
    assert_eq!(first_chunk.ty(), ChunkType::First);
    assert_eq!(last_chunk.ty(), ChunkType::Last);

    let mut bytes = BytesMut::new();

    for record_chunk in chunks.iter() {
        if let RecordChunk::Normal(chunk) = record_chunk {
            assert!(chunk.len() <= 4 * 1024);
            for slice in chunk.data.iter() {
                bytes.extend_from_slice(*slice)
            }
        }
    }

    let mut iter = chunks.iter();
    iter.next();
    let mut sum = 0;
    for item in iter {
        let size = match item {
            RecordChunk::Normal(c) => c.len(),
            RecordChunk::Slop(s) => *s,
        };

        sum += size;

        assert!(sum <= 4 * 1024);

        sum %= 4 * 1024;
    }

    let decode_record: Record = bytes.as_ref().into();

    assert_eq!(decode_record.value_size(), record.value_size());
    assert_eq!(decode_record.key_size(), record.key_size());
    assert_eq!(decode_record.len(), record.len());
}

#[test]
fn test_log_manager() {
    let mut tmp_dir = temp_dir();
    tmp_dir.push(format!("komachi_test_log_{}", random::<u16>()));
    create_dir(tmp_dir.clone()).unwrap();

    let work_dir = tmp_dir.as_path();

    let manager = LogManager::new(work_dir, 0, 4 * 1024).unwrap();
    manager.freeze_current_file(1).unwrap();

    let value = create_random_bytes(10000);

    let keys: Vec<u32> = (0..1500).collect();

    for key in keys.iter() {
        manager
            .insert_record(Record::new(key.to_ne_bytes().as_slice(), value.as_ref()))
            .unwrap();
    }

    manager.freeze_current_file(2).unwrap();

    let iter = manager.log_iterator(1).unwrap();

    let result_keys = iter
        .map(|item| {
            item.map(|wrapper| {
                let record = wrapper.record();
                let array: [u8; 4] = record.key().try_into().unwrap();
                u32::from_ne_bytes(array)
            })
        })
        .collect::<Result<Vec<u32>>>()
        .unwrap();

    assert_eq!(result_keys, keys);

    manager.truncate_log(0).unwrap();
    assert_eq!(manager.get_exist_log_number().unwrap(), vec![1]);
}
