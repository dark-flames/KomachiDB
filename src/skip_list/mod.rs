use crate::interface::Key;
use crate::skip_list::node::Node;

mod node;

#[allow(dead_code)]
pub struct SkipList<'pool, K: Key> {
    nodes: Vec<Node<'pool, K>>,
    entry: Option<&'pool Node<'pool, K>>,
    size: usize,
    max_level: usize,
}

#[allow(dead_code)]
impl<'pool, K: Key> SkipList<'pool, K> {
    pub fn new(max_level: usize) -> SkipList<'static, K> {
        SkipList {
            nodes: vec![],
            entry: None,
            size: 0,
            max_level,
        }
    }
}
