use crate::block::Block;
use crate::interface::Key;
use std::cmp::Ordering;

#[allow(dead_code)]
pub struct Node<'pool, K: Key> {
    pub block_ref: &'pool Block<'pool, K>,
    pub key: &'pool K,
    pub next: Vec<&'pool Node<'pool, K>>,
}

#[allow(dead_code)]
impl<'pool, K: Key> Node<'pool, K> {
    pub fn new(block: &'pool Block<'pool, K>) -> Node<'pool, K> {
        Node {
            block_ref: block,
            key: <&K>::clone(&block.key),
            next: vec![],
        }
    }

    pub fn add_level(&mut self, next: &'pool Node<'pool, K>) {
        self.next.push(next)
    }
}

impl<'pool, K: Key> PartialEq for Node<'pool, K> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<'pool, K: Key> PartialOrd for Node<'pool, K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.key.partial_cmp(other.key)
    }
}
