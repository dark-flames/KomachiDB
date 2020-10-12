use std::fmt::Display;

pub trait Key: PartialOrd + Copy + Display {
    fn from_bytes(bytes: &[u8]) -> Self;
}
