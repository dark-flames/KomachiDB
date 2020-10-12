use crate::interface::Key;
use crate::skip_list::iter::{SkipListIterator, SkipListVisitor};
use crate::skip_list::level_generator::LevelGenerator;
use crate::skip_list::node::Node;
use std::cmp::Ordering;

mod iter;
mod level_generator;
mod node;

#[allow(dead_code)]
pub struct SkipList<K: Key> {
    entry: *mut Node<K>,
    size: usize,
    level_generator: Box<dyn LevelGenerator>,
}

#[allow(dead_code)]
impl<K: Key> SkipList<K> {
    pub fn new(level_generator: Box<dyn LevelGenerator>) -> SkipList<K> {
        let head = Box::new(Node::head());
        SkipList {
            entry: Box::into_raw(head),
            size: 0,
            level_generator,
        }
    }

    pub fn insert(&mut self, key: K, ptr: *const [u8]) {
        let node = Box::into_raw(Box::new(Node::new(key, ptr)));
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
                let next = (*prev_node).next.get(level).copied();
                if let Some(next_node) = next {
                    (*node).add_level(next_node);
                    (*prev_node).next[level] = node;
                } else {
                    (*prev_node).next.push(node);
                }
            };
        }

        self.size += 1;
    }

    pub fn seek(&self, key: &K) -> Option<*const [u8]> {
        let mut iter = self.visitor();

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
                Some(next) if next.compare_key(key) == Ordering::Equal => break next.ptr,
                _ => break None,
            };
        }
    }

    pub fn iter(&self) -> SkipListIterator<K> {
        self.visitor().into()
    }

    fn find_position(&mut self, node: *mut Node<K>) -> Vec<*mut Node<K>> {
        let mut prev = vec![];
        let mut iter = self.visitor();

        loop {
            let next = iter.peek();

            match next {
                Some(next_node) if next_node < node => {
                    iter.next();
                }
                _ => {
                    if let Some(next_node) = next {
                        if next_node < node {
                            prev.push(iter.current().unwrap());
                        }
                    };

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

    fn visitor(&self) -> SkipListVisitor<K> {
        SkipListVisitor::new(self.entry, self.max_level())
    }
}

#[cfg(test)]
mod test {
    use crate::memory_pool::MemoryPool;
    use crate::skip_list::level_generator::RandomLevelGenerator;
    use crate::skip_list::SkipList;

    fn create_skip_list() -> SkipList<u32> {
        let level_generator = RandomLevelGenerator::new(10, 0.5);

        return SkipList::new(Box::new(level_generator));
    }

    fn create_pool() -> MemoryPool {
        MemoryPool::new(4096)
    }

    #[test]
    fn test_insert() {
        let mut pool = create_pool();

        let mut skip_list = create_skip_list();

        skip_list.insert(3, pool.allocate(4));
        skip_list.insert(2, pool.allocate(4));
        skip_list.insert(1, pool.allocate(4));

        assert!(skip_list.seek(&1).is_some());
        assert!(skip_list.seek(&2).is_some());
        assert!(skip_list.seek(&3).is_some());
        assert!(skip_list.seek(&4).is_none());
    }
}
