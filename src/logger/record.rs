use crate::format::{decode_usize, encode_usize};
use crate::logger::block::{Chunk, ChunkType, MAX_CHUNK_DATA_SIZE};
use crate::memtable::InternalKey;
use bytes::{Bytes, BytesMut};
use std::ptr::slice_from_raw_parts;

#[allow(dead_code)]
pub struct Record<'a> {
    key_size: Box<[u8]>,
    key: Bytes,
    value_size: Box<[u8]>,
    value: &'a [u8],
    pos: usize,
}

#[allow(dead_code)]
impl<'a> Record<'a> {
    pub fn new(key: &InternalKey, value: &'a [u8]) -> Record<'a> {
        let key_byte = key.as_bytes();
        Record {
            key_size: encode_usize(key_byte.len()),
            key: key_byte,
            value_size: encode_usize(value.len()),
            value,
            pos: 0,
        }
    }

    fn get_data(&self) -> Vec<&[u8]> {
        vec![
            self.key_size.as_ref(),
            self.key.as_ref(),
            self.value_size.as_ref(),
            self.value,
        ]
    }

    fn get_next_part(&self) -> Option<&[u8]> {
        let mut sum = 0;
        let mut data_iter = self.get_data().into_iter();

        loop {
            let part = match data_iter.next() {
                Some(p) => p,
                _ => break None,
            };

            if sum + part.len() >= self.pos {
                let offset = self.pos - sum;

                break Some(part.split_at(offset).1);
            } else {
                sum += part.len();
            }
        }
    }

    fn len(&self) -> usize {
        self.get_data()
            .into_iter()
            .fold(0, |carry, item| carry + item.len())
    }

    pub fn get_next_chunk(&mut self) -> Option<Chunk> {
        let mut data = BytesMut::new();

        let ty = loop {
            let part = match self.get_next_part() {
                Some(p) => p,
                _ => {
                    break if self.len() <= MAX_CHUNK_DATA_SIZE {
                        ChunkType::Full
                    } else {
                        ChunkType::Last
                    }
                }
            };

            if data.len() + part.len() <= MAX_CHUNK_DATA_SIZE {
                data.extend_from_slice(part);
                self.pos += part.len();
            } else {
                let size = MAX_CHUNK_DATA_SIZE - data.len();
                data.extend_from_slice(part.split_at(size).0);

                self.pos += size;
                break if self.pos == size {
                    ChunkType::First
                } else {
                    ChunkType::Middle
                };
            }
        };

        if data.is_empty() {
            None
        } else {
            Some(Chunk::new(data.freeze(), ty))
        }
    }
}

impl<'a> From<Vec<Chunk>> for Record<'a> {
    fn from(chunks: Vec<Chunk>) -> Self {
        let bytes = chunks
            .into_iter()
            .fold(BytesMut::new(), |mut carry, chunk| {
                carry.extend_from_slice(chunk.data.as_ref());
                carry
            })
            .freeze();

        let (key_size, key_size_right_ptr) = decode_usize(bytes.as_ptr());
        let (key_ref, value_size_ptr) = unsafe {
            (
                slice_from_raw_parts(key_size_right_ptr, key_size)
                    .as_ref()
                    .unwrap(),
                key_size_right_ptr.add(key_size),
            )
        };

        let key = Bytes::copy_from_slice(key_ref);
        let (value_size, value_ptr) = decode_usize(value_size_ptr);
        let value_ref = unsafe {
            slice_from_raw_parts(value_ptr, value_size)
                .as_ref()
                .unwrap()
        };

        Record {
            key_size: encode_usize(key_size),
            key,
            value_size: encode_usize(value_size),
            value: value_ref,
            pos: 0,
        }
    }
}
