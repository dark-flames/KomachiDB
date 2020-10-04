use crate::block::Block;
use crate::interface::Key;

#[allow(dead_code)]
pub struct Node<'pool, K: Key> {
    block_ref: &'pool Block<'pool, K>,
    key: &'pool K,
    next: Vec<&'pool Node<'pool, K>>,
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
