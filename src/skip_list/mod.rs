use crate::interface::Key;
use crate::skip_list::node::Node;

mod node;

#[allow(dead_code)]
pub struct SkipList<'pool, K: Key> {
    nodes: Vec<Node<'pool, K>>,
    entry: &'pool Node<'pool, K>,
    size: usize,
    max_level: usize,
}

#[allow(dead_code)]
impl<'pool, K: Key> SkipList<'pool, K> {}
