use bytes::{Bytes, BytesMut};
use crc32fast::Hasher;
use std::convert::TryInto;
use std::mem::size_of;

pub const BLOCK_SIZE: usize = 32 * 1024;
pub const MAX_CHUNK_DATA_SIZE: usize = BLOCK_SIZE
    - size_of::<u8>() // ty
    - size_of::<u32>() // crc
    - size_of::<u16>(); // size

pub enum ChunkType {
    Full,
    First,
    Middle,
    Last,
}

impl Into<u8> for ChunkType {
    fn into(self) -> u8 {
        match self {
            ChunkType::Full => 0,
            ChunkType::First => 1,
            ChunkType::Middle => 2,
            ChunkType::Last => 3,
        }
    }
}

impl From<&u8> for ChunkType {
    fn from(byte: &u8) -> Self {
        match byte {
            0 => ChunkType::Full,
            1 => ChunkType::First,
            2 => ChunkType::Middle,
            3 => ChunkType::Last,
            _ => panic!("Unexpected chunk type"),
        }
    }
}

pub struct Chunk {
    pub ty: ChunkType,
    pub data: Bytes,
    crc32: u32,
}

#[allow(dead_code)]
impl Chunk {
    pub fn new(data: Bytes, ty: ChunkType) -> Self {
        assert!(data.len() <= MAX_CHUNK_DATA_SIZE);

        let mut hasher = Hasher::new();
        hasher.update(data.as_ref());

        Chunk {
            ty,
            crc32: hasher.finalize(),
            data,
        }
    }

    pub fn check_crc32(&self) -> bool {
        let mut hasher = Hasher::new();
        hasher.update(self.data.as_ref());

        hasher.finalize() == self.crc32
    }
}

impl Into<Bytes> for Chunk {
    fn into(self) -> Bytes {
        let mut bytes = BytesMut::from(self.crc32.to_be_bytes().as_slice());

        bytes.extend_from_slice((self.data.len() as u16).to_be_bytes().as_slice());
        let ty: u8 = self.ty.into();
        bytes.extend_from_slice(ty.to_be_bytes().as_slice());
        bytes.extend_from_slice(self.data.as_ref());

        assert!(bytes.len() <= BLOCK_SIZE);

        bytes.freeze()
    }
}

impl From<&[u8]> for Chunk {
    fn from(bytes: &[u8]) -> Self {
        let (crc_bytes, crc_right) = bytes.split_at(size_of::<u32>());
        let crc32 = u32::from_be_bytes(crc_bytes.try_into().unwrap());
        let (size_bytes, size_right) = crc_right.split_at(size_of::<u16>());
        let size = u16::from_be_bytes(size_bytes.try_into().unwrap());
        let (ty_byte, data) = size_right.split_at(size_of::<u8>());
        let ty = ty_byte.first().unwrap().into();

        Chunk {
            ty,
            crc32,
            data: Bytes::copy_from_slice(data.split_at(size as usize).0),
        }
    }
}
