use crate::block::Block;
use crate::interface::Key;
use crate::skip_list::level_generator::LevelGenerator;
use crate::skip_list::node::Node;

mod level_generator;
mod node;

#[allow(dead_code)]
pub struct SkipList<'pool, K: Key> {
    entry: *mut Node<'pool, K>,
    size: usize,
    level_generator: &'static dyn LevelGenerator,
}

#[allow(dead_code)]
impl<'pool, K: Key> SkipList<'pool, K> {
    pub fn new(level_generator: &'static dyn LevelGenerator) -> SkipList<'static, K> {
        let head = Box::new(Node::head());
        SkipList {
            entry: Box::into_raw(head),
            size: 0,
            level_generator,
        }
    }

    pub fn insert(&mut self, block: &'pool Block<K>) {
        let node = Box::into_raw(Box::new(Node::new(block)));
        let level = self.level_generator.generate_level();

        let mut prev = self.find_position(node);

        while prev.len() > level + 1 {
            prev.pop();
        }

        while prev.len() < level + 1 {
            prev.push(self.entry);
        }

        for (level, prev_node) in prev.into_iter().enumerate() {
            unsafe {
                let next = (*prev_node).next[level];
                (*node).add_level(next);
                (*prev_node).next[level] = node;
            };
        }
    }

    fn find_position(&mut self, node: *mut Node<'pool, K>) -> Vec<*mut Node<'pool, K>> {
        let mut prev = vec![];
        let mut current = self.entry;
        let mut current_level = self.max_level();

        loop {
            let next = unsafe { (*current).next.get(current_level).copied() };

            match next {
                Some(next_node) if next_node < node => {
                    current = next_node;
                }
                _ => {
                    prev.push(current);

                    if current_level == 0 {
                        break;
                    } else {
                        current_level -= 1;
                        current = unsafe { (*current).next[current_level] };
                    }
                }
            }
        }

        prev.reverse();

        prev
    }

    fn max_level(&self) -> usize {
        match unsafe { (*self.entry).next.len() } {
            0 => 0,
            n => n - 1,
        }
    }
}
