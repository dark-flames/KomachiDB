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
    #[error("Unexpected chunk CRC code")]
    UnexpectedChunkCRC,
}

#[macro_export]
macro_rules! assert_as_error {
    ($input: expr, $error: expr) => {
        if ($input) { Ok(()) } else { Err($error) }?
    };
}
