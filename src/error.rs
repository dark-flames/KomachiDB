use std::result::Result as StdResult;
use thiserror::Error;

pub type Result<T> = StdResult<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Sequence number overflow")]
    SequenceNumberOverflow,
}

#[macro_export]
macro_rules! assert_as_error {
    ($input: expr, $error: expr) => {
        if ($input) { Ok(()) } else { Err($error) }?
    };
}
