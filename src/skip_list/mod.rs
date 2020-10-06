use crate::interface::Key;
use crate::skip_list::level_generator::LevelGenerator;
use crate::skip_list::node::SkipListNode;

mod level_generator;
mod node;

#[allow(dead_code)]
pub struct SkipList<'pool, K: Key> {
    entry: Box<SkipListNode<'pool, K>>,
    size: usize,
    level_generator: &'static dyn LevelGenerator,
}

#[allow(dead_code)]
impl<'pool, K: Key> SkipList<'pool, K> {
    pub fn new(level_generator: &'static dyn LevelGenerator) -> SkipList<'static, K> {
        SkipList {
            entry: Box::new(SkipListNode::Head),
            size: 0,
            level_generator,
        }
    }
}
