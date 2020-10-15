use crate::comparator::Comparator;
use std::cmp::Ordering;

pub struct NumberComparator<T: Sized + PartialOrd>(*const T);

impl<T: 'static + Sized + PartialOrd + Copy> Comparator for NumberComparator<T> {
    fn compare(a: &[u8], b: &[u8]) -> Ordering {
        let a_ref = unsafe { (a.as_ptr() as *const T).as_ref().unwrap() };

        let b_ref = unsafe { (b.as_ptr() as *const T).as_ref().unwrap() };

        a_ref.partial_cmp(b_ref).unwrap()
    }
}
