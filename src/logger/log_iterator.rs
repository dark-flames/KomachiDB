use crate::error::{Error, Result};
use crate::logger::chunk::{Chunk, MIN_CHUNK_SIZE};
use crate::logger::record::Record;
use std::fs::File;
use std::io::Read;

pub struct RecordWrapper {
    chunk_data: Vec<u8>,
}

#[allow(dead_code)]
impl RecordWrapper {
    pub fn record(&self) -> Record {
        Record::from(self.chunk_data.as_slice())
    }
}

pub struct LogIterator {
    file: File,
    file_name: String,
    block_size: usize,
    suffix: Vec<u8>,
    end_of_file: bool,
}

impl LogIterator {
    pub fn new(file_name: String, block_size: usize, file: File) -> Self {
        LogIterator {
            file,
            file_name,
            block_size,
            suffix: vec![],
            end_of_file: false,
        }
    }
}

impl Iterator for LogIterator {
    type Item = Result<RecordWrapper>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut data = vec![];

        if self.end_of_file && self.suffix.len() < MIN_CHUNK_SIZE {
            return None;
        }

        loop {
            let mut block = vec![0; self.block_size];
            let mut block_ref = if self.suffix.len() >= MIN_CHUNK_SIZE {
                self.suffix.as_slice()
            } else {
                let bytes = match self.file.read(&mut block[..]) {
                    Ok(bytes) => bytes,
                    Err(_) => return Some(Err(Error::UnableToReadLogFile(self.file_name.clone()))),
                };

                if bytes != self.block_size {
                    self.end_of_file = true
                }

                block.as_slice().split_at(bytes).0
            };

            let find_ending_chunk = loop {
                let chunk: Chunk = block_ref.into();

                if !chunk.check_crc32() {
                    return Some(Err(Error::UnexpectedChunkCRC(self.file_name.clone())));
                }

                let ty = chunk.ty();
                let size = chunk.len();
                data.extend_from_slice(chunk.data.first().cloned().unwrap());

                let left = block_ref.len() - size;
                block_ref = block_ref.split_at(size).1;

                if left < MIN_CHUNK_SIZE {
                    break ty.is_ending();
                } else if ty.is_ending() {
                    break true;
                }
            };

            if block_ref.len() >= MIN_CHUNK_SIZE {
                self.suffix = Vec::from(block_ref);
            } else {
                self.suffix = vec![];
            }

            if find_ending_chunk {
                break;
            }
        }

        Some(Ok(RecordWrapper { chunk_data: data }))
    }
}
