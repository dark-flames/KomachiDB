use crate::error::{Error, Result};
use std::fs::File;
use std::path::Path;
use std::sync::RwLock;

pub type LogNumber = u64;

pub struct LogManager {
    dir: Box<Path>,
    current_file: RwLock<File>,
}

impl LogManager {
    pub fn freeze_current_file(&self, new_log_number: LogNumber) -> Result<()> {
        let mut old_guard = self.current_file.write().unwrap();
        *old_guard =
            File::create(self.dir.join(format!("log_{}", new_log_number))).map_err(|_| {
                Error::UnableToCreateFile(self.dir.as_os_str().to_str().unwrap().to_string())
            })?;

        Ok(())
    }
}
