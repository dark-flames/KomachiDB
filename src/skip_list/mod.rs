use crate::block::Block;
use crate::interface::Key;
use crate::skip_list::iter::SkipListInternalIterator;
use crate::skip_list::level_generator::LevelGenerator;
use crate::skip_list::node::Node;
use std::cmp::Ordering;

mod iter;
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

    pub fn seek(&self, key: &K) -> Option<&Block<K>> {
        let mut iter = self.internal_iter();

        loop {
            let next_option = loop {
                if iter.current_level() == 0 {
                    break iter.peek_as_ref();
                };

                match iter.peek_as_ref() {
                    Some(next) if next.compare_key(key) == Ordering::Greater => {
                        iter.next_level();
                    }
                    Some(next) => {
                        break Some(next);
                    }
                    None => {
                        iter.next_level();
                    }
                };
            };

            match next_option {
                Some(next) if next.compare_key(key) == Ordering::Less => iter.next(),
                Some(next) if next.compare_key(key) == Ordering::Equal => {
                    break Some(next.block.unwrap())
                }
                _ => None,
            };
        }
    }

    fn find_position(&mut self, node: *mut Node<'pool, K>) -> Vec<*mut Node<'pool, K>> {
        let mut prev = vec![];
        let mut iter = self.internal_iter();

        loop {
            let next = iter.peek();

            match next {
                Some(next_node) if next_node < node => {
                    iter.next();
                }
                _ => {
                    prev.push(iter.current().unwrap());

                    if iter.current_level() == 0 {
                        break;
                    } else {
                        iter.next_level();
                        iter.next();
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

    fn internal_iter(&self) -> SkipListInternalIterator<'pool, K> {
        SkipListInternalIterator::new(self.entry, self.max_level())
    }
}
