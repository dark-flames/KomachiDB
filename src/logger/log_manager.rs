use crate::error::{Error, Result};
use std::fs::{remove_file, File};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

pub type LogNumber = u64;

pub struct LogManager {
    dir: Box<Path>,
    current_file: RwLock<File>,
}

#[allow(dead_code)]
impl LogManager {
    fn log_file(&self, log_number: LogNumber) -> PathBuf {
        self.dir.join(format!("log_{}", log_number))
    }

    pub fn freeze_current_file(&self, new_log_number: LogNumber) -> Result<()> {
        let mut old_guard = self.current_file.write().unwrap();
        *old_guard = File::create(self.log_file(new_log_number)).map_err(|_| {
            Error::UnableToCreateFile(self.dir.as_os_str().to_str().unwrap().to_string())
        })?;

        Ok(())
    }

    pub fn truncate_log(&self, log_number: LogNumber) -> Result<()> {
        let file_path = self.log_file(log_number);
        let path_str = file_path.to_str().unwrap().to_string();

        remove_file(file_path).map_err(|_| Error::UnableToTruncateLogFile(path_str))?;

        Ok(())
    }
}
