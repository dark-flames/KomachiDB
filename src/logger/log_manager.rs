use crate::error::{Error, Result};
use crate::logger::block::BLOCK_SIZE;
use crate::logger::record::Record;
use std::fs::{remove_file, File};
use std::io::{IoSlice, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;

pub type LogNumber = u64;

pub struct LogManager {
    dir: Box<Path>,
    current_log_number: AtomicU64,
    current_file: Mutex<File>,
    remaining_size: AtomicUsize,
}

#[allow(dead_code)]
impl LogManager {
    fn log_file(&self, log_number: LogNumber) -> PathBuf {
        self.dir.join(format!("log_{}", log_number))
    }

    fn current_log_file(&self) -> PathBuf {
        self.log_file(self.current_log_number.load(Ordering::SeqCst))
    }

    pub fn freeze_current_file(&self, new_log_number: LogNumber) -> Result<()> {
        let mut old_guard = self.current_file.lock().unwrap();
        *old_guard = File::create(self.log_file(new_log_number)).map_err(|_| {
            Error::UnableToCreateFile(self.dir.as_os_str().to_str().unwrap().to_string())
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
        let mut buffer = self.current_file.lock().unwrap();

        let (chunks, remaining_size) =
            record.get_chunks(self.remaining_size.load(Ordering::SeqCst), BLOCK_SIZE);

        self.remaining_size.store(remaining_size, Ordering::SeqCst);

        let slices: Vec<_> = chunks
            .iter()
            .map::<Vec<&[u8]>, _>(|chunk| chunk.into())
            .flatten()
            .map(|slice| IoSlice::new(slice))
            .collect();

        match buffer.write_vectored(slices.as_ref()) {
            Err(_) => Err(Error::UnableToWriteLogFile(
                self.current_log_file().to_str().unwrap().to_string(),
            )),
            _ => match buffer.flush() {
                Err(_) => Err(Error::UnableToWriteLogFile(
                    self.current_log_file().to_str().unwrap().to_string(),
                )),
                _ => Ok(()),
            },
        }
    }
}
