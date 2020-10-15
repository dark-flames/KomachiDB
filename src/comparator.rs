use std::cmp::Ordering;

pub trait Comparator {
    fn compare(a: &[u8], b: &[u8]) -> Ordering;
}
