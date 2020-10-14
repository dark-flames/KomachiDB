use crate::interface::Key;
use std::cmp::Ordering;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Node<K: Key> {
    pub ptr: Option<*const [u8]>,
    pub key: Option<*const K>,
    pub next: Vec<*mut Node<K>>,
}

#[allow(dead_code)]
impl<K: Key> Node<K> {
    pub fn new(key: *const K, ptr: *const [u8]) -> Node<K> {
        Node {
            key: Some(key),
            ptr: Some(ptr),
            next: vec![],
        }
    }

    pub fn head() -> Node<K> {
        Node {
            key: None,
            ptr: None,
            next: vec![],
        }
    }

    pub fn is_head(&self) -> bool {
        self.key.is_none()
    }

    pub fn add_level(&mut self, next: *mut Node<K>) {
        self.next.push(next)
    }

    pub fn compare_key(&self, key: &K) -> Ordering {
        self.key_ref().map_or(Ordering::Less, |self_key| {
            self_key.partial_cmp(key).unwrap()
        })
    }

    pub fn key_ref(&self) -> Option<&K> {
        unsafe { self.key.map(|ptr| ptr.as_ref()).flatten() }
    }
}

impl<K: Key> PartialEq for Node<K> {
    fn eq(&self, other: &Self) -> bool {
        match (&self.key, &other.key) {
            (Some(self_key), Some(other_key)) => self_key == other_key,
            (None, None) => true,
            _ => false,
        }
    }
}

impl<K: Key> PartialOrd for Node<K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self.key_ref(), other.key_ref()) {
            (Some(self_key), Some(other_key)) => self_key.partial_cmp(other_key),
            (None, Some(_)) => Some(Ordering::Less),
            (Some(_), None) => Some(Ordering::Greater),
            (None, None) => Some(Ordering::Equal),
        }
    }
}
