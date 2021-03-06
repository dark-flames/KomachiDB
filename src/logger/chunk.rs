use crc32fast::Hasher;
use std::convert::TryInto;
use std::mem::size_of;

pub const CHUNK_HEAD_SIZE: usize = size_of::<u8>() // ty
    + size_of::<u32>() // crc
    + size_of::<u16>();

pub const MIN_CHUNK_SIZE: usize = CHUNK_HEAD_SIZE * 2;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ChunkType {
    Full,
    First,
    Middle,
    Last,
}

impl ChunkType {
    pub fn is_ending(&self) -> bool {
        matches!(self, ChunkType::Full | ChunkType::Last)
    }
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

pub struct Chunk<'a> {
    pub data: Vec<&'a [u8]>,
    ty: [u8; 1],
    data_size: [u8; 2],
    crc32: [u8; 4],
}

#[allow(dead_code)]
impl<'a> Chunk<'a> {
    pub fn new(data: Vec<&'a [u8]>, ty: ChunkType) -> Self {
        let mut hasher = Hasher::new();
        for item in data.clone() {
            hasher.update(item);
        }

        let data_size = data.iter().fold(0, |carry, item| carry + item.len()) as u16;

        Chunk {
            ty: [ty.into()],
            crc32: hasher.finalize().to_ne_bytes(),
            data_size: data_size.to_ne_bytes(),
            data,
        }
    }

    pub fn check_crc32(&self) -> bool {
        let mut hasher = Hasher::new();
        for item in self.data.clone() {
            hasher.update(item);
        }

        hasher.finalize() == u32::from_ne_bytes(self.crc32)
    }

    pub fn len(&self) -> usize {
        self.data_len() as usize + CHUNK_HEAD_SIZE // size
    }

    pub fn data_len(&self) -> u16 {
        u16::from_ne_bytes(self.data_size)
    }

    pub fn ty(&self) -> ChunkType {
        ChunkType::from(&self.ty[0])
    }

    pub fn crc32(&self) -> u32 {
        u32::from_ne_bytes(self.crc32)
    }
}

impl<'a> AsRef<Chunk<'a>> for Chunk<'a> {
    fn as_ref(&self) -> &Chunk<'a> {
        self
    }
}

impl<'a, 'b> Into<Vec<&'b [u8]>> for &'b Chunk<'a> {
    fn into(self) -> Vec<&'b [u8]> {
        let mut slices = vec![
            self.crc32.as_slice(),
            self.data_size.as_slice(),
            self.ty.as_slice(),
        ];

        slices.extend(self.data.clone());

        slices
    }
}

impl<'a> From<&'a [u8]> for Chunk<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        assert!(bytes.len() >= MIN_CHUNK_SIZE);
        let (crc_bytes, crc_right) = bytes.split_at(size_of::<u32>());
        let crc32 = crc_bytes.try_into().unwrap();
        let (size_bytes, size_right) = crc_right.split_at(size_of::<u16>());
        let size: [u8; 2] = size_bytes.try_into().unwrap();
        let (ty_byte, data_right) = size_right.split_at(size_of::<u8>());
        if u16::from_ne_bytes(size) as usize > data_right.len() {
            panic!(format!(
                "data_right: {}, size: {}",
                data_right.len(),
                u16::from_ne_bytes(size)
            ))
        }
        let data = data_right.split_at(u16::from_ne_bytes(size) as usize).0;
        let ty = ty_byte.try_into().unwrap();

        Chunk {
            ty,
            crc32,
            data: vec![data],
            data_size: (data.len() as u16).to_ne_bytes(),
        }
    }
}
