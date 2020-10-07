use crate::block::Block;
use crate::interface::Key;
use std::cmp::Ordering;

#[allow(dead_code)]
pub struct Node<'pool, K: Key> {
    pub block: Option<&'pool Block<'pool, K>>,
    pub next: Vec<*mut Node<'pool, K>>,
}

#[allow(dead_code)]
impl<'pool, K: Key> Node<'pool, K> {
    pub fn new(block: &'pool Block<'pool, K>) -> Node<'pool, K> {
        Node {
            block: Some(block),
            next: vec![],
        }
    }

    pub fn head() -> Node<'pool, K> {
        Node {
            block: None,
            next: vec![],
        }
    }

    pub fn is_head(&self) -> bool {
        self.block.is_none()
    }

    pub fn add_level(&mut self, next: *mut Node<'pool, K>) {
        self.next.push(next)
    }
}

impl<'pool, K: Key> PartialEq for Node<'pool, K> {
    fn eq(&self, other: &Self) -> bool {
        match (self.block, other.block) {
            (Some(self_block), Some(other_block)) => self_block.key == other_block.key,
            (None, None) => true,
            _ => false,
        }
    }
}

impl<'pool, K: Key> PartialOrd for Node<'pool, K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self.block, other.block) {
            (Some(self_block), Some(other_block)) => self_block.key.partial_cmp(other_block.key),
            (None, Some(_)) => Some(Ordering::Less),
            (Some(_), None) => Some(Ordering::Greater),
            (None, None) => Some(Ordering::Equal),
        }
    }
}
