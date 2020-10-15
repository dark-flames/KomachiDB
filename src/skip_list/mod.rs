use crate::comparator::Comparator;
use crate::skip_list::iter::{SkipListIterator, SkipListVisitor};
use crate::skip_list::level_generator::LevelGenerator;
use crate::skip_list::node::Node;
use std::cmp::Ordering;

mod iter;
mod level_generator;
mod node;

#[allow(dead_code)]
pub struct SkipList<C: Comparator> {
    entry: *mut Node<C>,
    size: usize,
    level_generator: Box<dyn LevelGenerator>,
}

#[allow(dead_code)]
impl<C: Comparator> SkipList<C> {
    pub fn new(level_generator: Box<dyn LevelGenerator>) -> SkipList<C> {
        let head = Box::new(Node::head());
        SkipList {
            entry: Box::into_raw(head),
            size: 0,
            level_generator,
        }
    }

    pub fn insert(&mut self, key: *const [u8], ptr: *const u8) {
        let node_box = Box::new(Node::new(key, ptr));

        let level = self.level_generator.generate_level();

        let mut prev = self.find_position(node_box.as_ref());
        let node = Box::into_raw(Box::new(Node::new(key, ptr)));

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

    pub fn seek(&self, key: &[u8]) -> Option<*const u8> {
        let mut iter = self.visitor();

        loop {
            let next_option = loop {
                if iter.current_level() == 0 {
                    break iter.peek_as_ref();
                };

                match iter.peek_as_ref() {
                    Some(next) if next.compare_key(key) == Ordering::Less => {
                        break Some(next);
                    }
                    Some(_) | None => {
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

    pub fn iter(&self) -> SkipListIterator<C> {
        self.visitor().into()
    }

    fn find_position(&mut self, node: &Node<C>) -> Vec<*mut Node<C>> {
        let mut prev = vec![];
        let mut iter = self.visitor();

        loop {
            let next = iter.peek_as_ref();

            match next {
                Some(next_node) if next_node <= node => {
                    iter.next();
                }

                _ => {
                    if iter.current_as_ref().unwrap() < node {
                        prev.push(iter.current().unwrap());
                    }

                    if iter.current_level() == 0 {
                        break;
                    } else {
                        iter.next_level();
                    }
                }
            };
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

    fn visitor(&self) -> SkipListVisitor<C> {
        SkipListVisitor::new(self.entry, self.max_level())
    }
}

#[cfg(test)]
mod test {
    use crate::implement::NumberComparator;
    use crate::memory_pool::MemoryPool;
    use crate::skip_list::level_generator::RandomLevelGenerator;
    use crate::skip_list::SkipList;
    use rand::random;
    use std::collections::HashSet;
    use std::mem::size_of;
    use std::ptr::slice_from_raw_parts;

    fn create_skip_list() -> SkipList<NumberComparator<u32>> {
        let level_generator = RandomLevelGenerator::new(10, 0.5);

        return SkipList::new(Box::new(level_generator));
    }

    fn create_pool() -> MemoryPool {
        MemoryPool::new(4096)
    }

    fn get_number_ptr(n: u32) -> *const [u8] {
        slice_from_raw_parts(Box::into_raw(Box::new(n)) as *const u8, size_of::<u32>())
    }

    fn get_number_slice(n: u32) -> &'static [u8] {
        unsafe { get_number_ptr(n).as_ref().unwrap() }
    }

    #[test]
    fn simple_test_insert() {
        let mut pool = create_pool();

        let mut skip_list = create_skip_list();

        skip_list.insert(get_number_ptr(3), pool.allocate(4));
        skip_list.insert(get_number_ptr(2), pool.allocate(4));
        skip_list.insert(get_number_ptr(1), pool.allocate(4));

        assert!(skip_list.seek(get_number_slice(1)).is_some());
        assert!(skip_list.seek(get_number_slice(2)).is_some());
        assert!(skip_list.seek(get_number_slice(3)).is_some());
        assert!(skip_list.seek(get_number_slice(4)).is_none());
    }

    #[test]
    fn random_test_insert() {
        let mut pool = create_pool();
        let mut skip_list = create_skip_list();

        let mut set = HashSet::new();

        for _ in 0..100 {
            let key = loop {
                let result = random::<u32>();

                if !set.contains(&result) {
                    break result;
                }
            };

            skip_list.insert(get_number_ptr(key), pool.allocate(4));
            set.insert(key);
        }

        let mut set_vec = set.iter().map(|key| key.clone()).collect::<Vec<u32>>();
        set_vec.sort();
        assert_eq!(
            set_vec,
            skip_list
                .iter()
                .map(|(key, _)| unsafe { *(key.as_ptr() as *const u32) })
                .collect::<Vec<_>>()
        );

        for _ in 0..100 {
            let key = loop {
                let result = random::<u32>();

                if !set.contains(&result) {
                    break result;
                }
            };

            assert!(skip_list.seek(get_number_slice(key)).is_none());
        }
    }
}
