use crate::error::{Error, Result};
use crate::logger::block::Chunk;
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
}

impl LogIterator {
    pub fn new(file_name: String, block_size: usize, file: File) -> Self {
        LogIterator {
            file,
            file_name,
            block_size,
            suffix: vec![],
        }
    }
}

impl Iterator for LogIterator {
    type Item = Result<RecordWrapper>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut data = vec![];
        let first = true;

        loop {
            let mut block = vec![0; self.block_size];
            let (mut block_ref, ending) = if first {
                (self.suffix.as_slice(), false)
            } else {
                let ending = match self.file.read(&mut block) {
                    Ok(bytes) => bytes != self.block_size,
                    Err(_) => return Some(Err(Error::UnableToReadLogFile(self.file_name.clone()))),
                };

                (block.as_slice(), ending)
            };

            let find_ending_chunk = loop {
                let chunk: Chunk = block_ref.into();

                if !chunk.check_crc32() {
                    return Some(Err(Error::UnexpectedChunkCRC(self.file_name.clone())));
                }

                let ty = chunk.ty();
                let size = chunk.len();
                data.extend_from_slice(chunk.data.first().cloned().unwrap());

                if size == block_ref.len() {
                    break false;
                } else {
                    block_ref = block_ref.split_at(size).1;

                    if ty.is_ending() {
                        break true;
                    }
                }
            };

            if find_ending_chunk {
                if !block_ref.is_empty() {
                    self.suffix = Vec::from(block_ref);
                }

                break;
            }

            if ending {
                break;
            }
        }

        Some(Ok(RecordWrapper { chunk_data: data }))
    }
}
