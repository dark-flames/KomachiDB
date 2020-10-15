use crate::comparator::Comparator;
use std::cmp::Ordering;
use std::ptr::null;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Node<C: Comparator> {
    pub ptr: Option<*const u8>,
    pub key: Option<*const [u8]>,
    pub next: Vec<*mut Node<C>>,
    marker: *const C,
}

#[allow(dead_code)]
impl<C: Comparator> Node<C> {
    pub fn new(key: *const [u8], ptr: *const u8) -> Node<C> {
        Node {
            key: Some(key),
            ptr: Some(ptr),
            next: vec![],
            marker: null(),
        }
    }

    pub fn head() -> Node<C> {
        Node {
            key: None,
            ptr: None,
            next: vec![],
            marker: null(),
        }
    }

    pub fn is_head(&self) -> bool {
        self.key.is_none()
    }

    pub fn add_level(&mut self, next: *mut Node<C>) {
        self.next.push(next)
    }

    pub fn compare_key(&self, key: &[u8]) -> Ordering {
        self.key_ref()
            .map_or(Ordering::Less, |key_ref| C::compare(key_ref, key))
    }

    pub fn key_ref(&self) -> Option<&[u8]> {
        self.key
            .map(|key_ptr| unsafe { key_ptr.as_ref() })
            .flatten()
    }
}

impl<C: Comparator> PartialEq for Node<C> {
    fn eq(&self, other: &Self) -> bool {
        match (self.key_ref(), other.key_ref()) {
            (Some(self_key), Some(other_key)) => C::compare(self_key, other_key) == Ordering::Equal,
            (None, None) => true,
            _ => false,
        }
    }
}

impl<C: Comparator> PartialOrd for Node<C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self.key_ref(), other.key_ref()) {
            (Some(self_key), Some(other_key)) => Some(C::compare(self_key, other_key)),
            (None, Some(_)) => Some(Ordering::Less),
            (Some(_), None) => Some(Ordering::Greater),
            (None, None) => Some(Ordering::Equal),
        }
    }
}
