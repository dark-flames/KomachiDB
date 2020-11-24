use std::cmp::Ordering;
use std::fmt::Display;
use std::marker::PhantomData;

pub trait Comparator: Sync {
    fn compare(a: &[u8], b: &[u8]) -> Ordering;
}

pub struct NumberComparator<T: 'static + Sized + Ord + Copy + Display + Sync>(PhantomData<T>);

unsafe impl<T: 'static + Sized + Ord + Copy + Display + Sync> Sync for NumberComparator<T> {}

impl<T: 'static + Sized + Ord + Copy + Display + Sync> Comparator for NumberComparator<T> {
    fn compare(a: &[u8], b: &[u8]) -> Ordering {
        let a_ref = unsafe { (a.as_ptr() as *const T).as_ref().unwrap() };

        let b_ref = unsafe { (b.as_ptr() as *const T).as_ref().unwrap() };

        a_ref.cmp(b_ref)
    }
}
