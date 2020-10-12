use crate::interface::Key;
use std::convert::TryInto;

impl Key for u32 {
    fn from_bytes(bytes: &[u8]) -> Self {
        u32::from_be_bytes(bytes.try_into().unwrap())
    }
}
