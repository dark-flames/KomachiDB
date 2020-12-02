use crate::format::{decode_usize, encode_usize};
use crate::logger::chunk::{Chunk, ChunkType, CHUNK_HEAD_SIZE, MIN_CHUNK_SIZE};
use std::ptr::slice_from_raw_parts;

pub enum RecordChunk<'a> {
    Normal(Chunk<'a>),
    Slop(usize),
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Record<'a> {
    key_size: Box<[u8]>,
    key: &'a [u8],
    value_size: Box<[u8]>,
    value: &'a [u8],
}

#[allow(dead_code)]
impl<'a> Record<'a> {
    pub fn new(key: &'a [u8], value: &'a [u8]) -> Record<'a> {
        Record {
            key_size: encode_usize(key.len()),
            key,
            value_size: encode_usize(value.len()),
            value,
        }
    }

    pub fn key_size(&self) -> usize {
        decode_usize(self.key_size.as_ptr()).0
    }

    pub fn value_size(&self) -> usize {
        decode_usize(self.value.as_ptr()).0
    }

    pub fn len(&self) -> usize {
        self.get_data()
            .into_iter()
            .fold(0, |carry, item| carry + item.len())
    }

    pub fn key(&self) -> &[u8] {
        self.key
    }

    pub fn value(&self) -> &[u8] {
        self.value
    }

    fn get_data(&self) -> Vec<&[u8]> {
        let mut data = vec![self.key_size.as_ref()];

        if self.key_size() != 0 {
            data.push(self.key)
        }

        data.push(self.value_size.as_ref());

        if self.value_size() != 0 {
            data.push(self.value)
        }

        data
    }

    fn get_next_part(&self, pos: usize) -> Option<&[u8]> {
        let mut sum = 0;
        let mut data_iter = self.get_data().into_iter();

        loop {
            let part = match data_iter.next() {
                Some(p) => p,
                _ => break None,
            };

            if sum + part.len() > pos {
                let offset = pos - sum;

                break Some(part.split_at(offset).1);
            } else {
                sum += part.len();
            }
        }
    }

    fn get_next_chunk(&self, size: usize, mut pos: usize) -> Option<(Chunk, usize)> {
        assert!(size > MIN_CHUNK_SIZE);
        let mut data: Vec<&[u8]> = vec![];
        let mut data_size = 0;

        let chunk_data_size = size - CHUNK_HEAD_SIZE;

        let ty = loop {
            let part = match self.get_next_part(pos) {
                Some(p) => p,
                _ => {
                    break if self.len() <= chunk_data_size {
                        ChunkType::Full
                    } else {
                        ChunkType::Last
                    }
                }
            };

            if data_size + part.len() <= chunk_data_size {
                data.push(part);
                data_size += part.len();
                pos += part.len();
            } else {
                let part_size = chunk_data_size - data_size;
                data_size += part_size;
                data.push(part.split_at(part_size).0);

                pos += part_size;
                break if pos == data_size {
                    ChunkType::First
                } else {
                    ChunkType::Middle
                };
            }
        };

        if data.is_empty() {
            None
        } else {
            let chunk = Chunk::new(data, ty);

            Some((chunk, pos))
        }
    }

    pub fn get_chunks(&self, first_size: usize, max_size: usize) -> (Vec<RecordChunk>, usize) {
        let mut chunks = vec![];

        let mut left_size = first_size;

        let mut pos = 0;

        while let Some((c, new_pos)) = self.get_next_chunk(left_size, pos) {
            let len = c.len();
            assert!(len <= left_size);
            chunks.push(RecordChunk::Normal(c));

            left_size = match left_size - len {
                0 => max_size,
                others if others < MIN_CHUNK_SIZE => {
                    chunks.push(RecordChunk::Slop(others));
                    max_size
                }
                others => others,
            };

            pos = new_pos;
        }

        (chunks, left_size)
    }
}

impl<'a> From<&[u8]> for Record<'a> {
    fn from(slice: &[u8]) -> Self {
        let (key_size, key_size_right_ptr) = decode_usize(slice.as_ptr());
        let (key, value_size_ptr) = unsafe {
            (
                slice_from_raw_parts(key_size_right_ptr, key_size)
                    .as_ref()
                    .unwrap(),
                key_size_right_ptr.add(key_size),
            )
        };

        let (value_size, value_ptr) = decode_usize(value_size_ptr);
        let value = unsafe {
            slice_from_raw_parts(value_ptr, value_size)
                .as_ref()
                .unwrap()
        };

        Record {
            key_size: encode_usize(key_size),
            key,
            value_size: encode_usize(value_size),
            value,
        }
    }
}
