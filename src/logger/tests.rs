use crate::logger::block::{Chunk, ChunkType};
use crate::logger::record::Record;
use bytes::{Bytes, BytesMut};

#[test]
fn test_chunk_encode() {
    let data = Bytes::from("1145141919".to_string());
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
    let key = Bytes::from("114514");
    let value = Bytes::from("iiyo koiyo");

    let record = Record::new(key.as_ref(), value.as_ref());
    let chunks = record.get_chunks(4 * 1024, 4 * 1024).0;
    let first_chunk = chunks.first().unwrap();

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
    let key = Bytes::from("114514");
    let value = Bytes::from("iiyo koiyo".repeat(10000));

    let record = Record::new(key.as_ref(), value.as_ref());
    let chunks = record.get_chunks(1024, 4 * 1024).0;

    assert_eq!(chunks.first().unwrap().len(), 1024);
    assert_eq!(chunks.first().unwrap().ty(), ChunkType::First);
    assert_eq!(chunks.last().unwrap().ty(), ChunkType::Last);

    let mut bytes = BytesMut::new();

    for chunk in chunks {
        assert!(chunk.len() <= 4 * 1024);
        for slice in chunk.data {
            bytes.extend_from_slice(slice)
        }
    }

    let decode_record: Record = bytes.as_ref().into();

    assert_eq!(decode_record.value_size(), record.value_size());
    assert_eq!(decode_record.key_size(), record.key_size());
    assert_eq!(decode_record.len(), record.len());
}
