use std::result::Result as StdResult;
use thiserror::Error;

pub type Result<T> = StdResult<T, Error>;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum Error {
    #[error("Sequence number overflow")]
    SequenceNumberOverflow,
    #[error("Unable to create file at \"{0}\"")]
    UnableToCreateFile(String),
    #[error("Unable to read dir \"{0}\"")]
    UnableToReadDir(String),
    #[error("Unable to truncate log file: \"{0}\"")]
    UnableToTruncateLogFile(String),
    #[error("Unable to write log file: \"{0}\"")]
    UnableToWriteLogFile(String),
    #[error("Unable to read log file: \"{0}\"")]
    UnableToReadLogFile(String),
    #[error("Unexpected chunk CRC code at file: \"{0}\"")]
    UnexpectedChunkCRC(String),
}

#[macro_export]
macro_rules! assert_as_error {
    ($input: expr, $error: expr) => {
        if ($input) { Ok(()) } else { Err($error) }?
    };
}
