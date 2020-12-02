use crate::error::{Error, Result};
use crate::logger::log_iterator::LogIterator;
use crate::logger::record::{Record, RecordChunk};
use regex::Regex;
use std::cmp::max;
use std::fs::{read_dir, remove_file, DirEntry, File};
use std::io::{Error as IOError, IoSlice, Write};
use std::path::{Path, PathBuf};
use std::result::Result as STDResult;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;

pub type LogNumber = u64;

pub struct LogManager {
    dir: PathBuf,
    current_log_number: AtomicU64,
    current_file: Mutex<File>,
    remaining_size: AtomicUsize,
    block_size: AtomicUsize,
}

// todo: use
#[allow(dead_code)]
impl LogManager {
    pub fn new(dir: PathBuf, first_log_number: LogNumber, block_size: usize) -> Result<Self> {
        Ok(LogManager {
            dir: dir.clone(),
            current_log_number: AtomicU64::new(first_log_number),
            current_file: Mutex::new(
                File::create(dir.join(format!("log_{}", first_log_number))).map_err(|_| {
                    Error::UnableToCreateFile(dir.as_os_str().to_str().unwrap().to_string())
                })?,
            ),
            remaining_size: AtomicUsize::new(block_size),
            block_size: AtomicUsize::new(block_size),
        })
    }

    fn dir(&self) -> &Path {
        self.dir.as_path()
    }

    fn log_file(&self, log_number: LogNumber) -> PathBuf {
        self.dir().join(format!("log_{}", log_number))
    }

    fn current_log_file(&self) -> PathBuf {
        self.log_file(self.current_log_number.load(Ordering::SeqCst))
    }

    pub fn freeze_current_file(&self, new_log_number: LogNumber) -> Result<()> {
        let mut old_guard = match self.current_file.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        *old_guard = File::create(self.log_file(new_log_number)).map_err(|_| {
            Error::UnableToCreateFile(self.dir().as_os_str().to_str().unwrap().to_string())
        })?;
        self.current_log_number
            .store(new_log_number, Ordering::SeqCst);

        Ok(())
    }

    pub fn truncate_log(&self, log_number: LogNumber) -> Result<()> {
        let file_path = self.log_file(log_number);
        let path_str = file_path.to_str().unwrap().to_string();

        remove_file(file_path).map_err(|_| Error::UnableToTruncateLogFile(path_str))?;

        Ok(())
    }

    pub fn insert_record(&self, record: Record) -> Result<()> {
        let mut buffer = match self.current_file.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        let (chunks, remaining_size) = record.get_chunks(
            self.remaining_size.load(Ordering::SeqCst),
            self.block_size.load(Ordering::SeqCst),
        );

        self.remaining_size.store(remaining_size, Ordering::SeqCst);

        let slop: Vec<u8> = vec![
            0;
            chunks.iter().fold(0, |carry, record_chunk| {
                max(
                    carry,
                    match record_chunk {
                        RecordChunk::Slop(size) => *size,
                        _ => 0,
                    },
                )
            })
        ];

        let mut slices = vec![];

        for chunk in chunks.iter() {
            match chunk {
                RecordChunk::Normal(c) => {
                    let chunk_slices: Vec<&[u8]> = c.into();
                    let mut sum = 0;
                    slices.extend(chunk_slices.into_iter().map(|slice| {
                        sum += slice.len();
                        IoSlice::new(slice)
                    }));
                }
                RecordChunk::Slop(size) => {
                    slices.push(IoSlice::new(slop.as_slice().split_at(*size).0))
                }
            }
        }

        match buffer.write_all_vectored(slices.as_mut_slice()) {
            Err(_) => Err(Error::UnableToWriteLogFile(
                self.current_log_file().to_str().unwrap().to_string(),
            )),
            Ok(_) => match buffer.flush() {
                Err(_) => Err(Error::UnableToWriteLogFile(
                    self.current_log_file().to_str().unwrap().to_string(),
                )),
                _ => Ok(()),
            },
        }
    }

    pub fn log_iterator(&self, log_number: LogNumber) -> Result<LogIterator> {
        let file_path = self.log_file(log_number);
        let file_name = file_path.to_str().unwrap().to_string();
        let file =
            File::open(file_path).map_err(|_| Error::UnableToReadLogFile(file_name.clone()))?;

        Ok(LogIterator::new(
            file_name,
            self.block_size.load(Ordering::SeqCst),
            file,
        ))
    }

    pub fn get_exist_log_number(&self) -> Result<Vec<LogNumber>> {
        let entries = read_dir(&self.dir())
            .map_err(|_| Error::UnableToReadDir(self.dir.to_str().unwrap().to_string()))?
            .collect::<STDResult<Vec<DirEntry>, IOError>>()
            .map_err(|_| Error::UnableToReadDir(self.dir.to_str().unwrap().to_string()))?;

        let current_log_number = self.current_log_number.load(Ordering::SeqCst);

        Ok(entries
            .into_iter()
            .filter_map(|entry| {
                let regex = Regex::new(r"^log_(\d+)$").unwrap();

                let result = regex
                    .captures(entry.file_name().to_str().unwrap())
                    .map(|result| {
                        result
                            .get(1)
                            .map(|num| num.as_str().parse::<LogNumber>().unwrap())
                    })
                    .flatten();

                match result {
                    Some(n) if n == current_log_number => None,
                    others => others,
                }
            })
            .collect())
    }
}
