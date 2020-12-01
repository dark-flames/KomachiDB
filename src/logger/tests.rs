use crate::logger::block::{Chunk, ChunkType};
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
}
