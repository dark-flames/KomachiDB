use crate::skip_list::arena::Arena;
use crate::skip_list::comparator::Comparator;
use crate::skip_list::iter::SkipListVisitor;
use crate::skip_list::level_generator::LevelGenerator;
use crate::skip_list::node::Node;
use bytes::Bytes;
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering as AtomicOrdering;

pub struct SkipList<C: Comparator> {
    entry: NonNull<Node>,
    arena: Arena<Node>,
    len: AtomicUsize,
    level_generator: Box<dyn LevelGenerator>,
    _key_comparator: PhantomData<C>,
}

#[allow(dead_code)]
impl<C: Comparator> SkipList<C> {
    pub fn new(arena_capacity: u32, level_generator: Box<dyn LevelGenerator>) -> SkipList<C> {
        let mut arena = Arena::with_capacity(arena_capacity);

        let entry_node = Node::allocate_with_arena(
            Bytes::new(),
            Bytes::new(),
            level_generator.max_level(),
            &mut arena,
        );

        let entry = unsafe { NonNull::new_unchecked(arena.get_mut(entry_node)) };

        SkipList {
            entry,
            arena,
            len: AtomicUsize::new(0),
            level_generator,
            _key_comparator: Default::default(),
        }
    }

    pub fn insert(&mut self, key: Bytes, value: Bytes) {
        let level = self.level_generator.generate_level();

        let node_offset = Node::allocate_with_arena(key, value, level, &mut self.arena);
        let node = unsafe { &mut *self.arena.get_mut(node_offset) };

        let mut prev = self.find_prev_offset(node);

        while prev.len() > level + 1 {
            prev.pop();
        }

        let entry_offset = self.arena.get_offset(self.entry.as_ptr());

        while prev.len() < level + 1 {
            prev.push(entry_offset);
        }

        for (level, prev_node_offset) in prev.into_iter().enumerate() {
            let prev_node = unsafe { &mut *self.arena.get_mut(prev_node_offset) };
            node.set_next(level, prev_node.next_offset(level));
            prev_node.set_next(level, node_offset);
        }

        self.len.fetch_add(1, AtomicOrdering::SeqCst);
    }

    pub fn seek(&self, key: &Bytes) -> Option<&Bytes> {
        match self.seek_offset(key) {
            0 => None,
            offset => Some(unsafe { &*self.arena.get(offset) }.value()),
        }
    }

    pub fn len(&self) -> usize {
        self.len.load(AtomicOrdering::SeqCst)
    }

    fn visitor(&self) -> SkipListVisitor<C> {
        SkipListVisitor::create(self.entry, self.height(), &self.arena)
    }

    fn seek_offset(&self, key: &Bytes) -> u32 {
        let mut visitor = self.visitor();

        loop {
            match visitor.compare_next_key(key) {
                Ordering::Less => {
                    visitor.next();
                }
                Ordering::Equal => {
                    break visitor.peek_offset().unwrap_or(0);
                }
                Ordering::Greater if visitor.current_level() == 0 => {
                    break 0;
                }
                Ordering::Greater => {
                    visitor.reduce_level();
                }
            }
        }
    }

    fn find_prev_offset(&mut self, node: &Node) -> Vec<u32> {
        let mut visitor = self.visitor();
        let mut prev = vec![];

        loop {
            if let Ordering::Less = visitor.compare_next_key(node.key()) {
                visitor.next();
            } else {
                if let Ordering::Less = visitor.compare_key(node.key()) {
                    prev.push(visitor.current_offset())
                }

                if visitor.current_level() == 0 {
                    break;
                } else {
                    visitor.reduce_level()
                }
            }
        }
        prev.reverse();

        prev
    }

    fn height(&self) -> usize {
        unsafe { self.entry.as_ref().height() }
    }
}

#[cfg(test)]
mod test {
    use crate::implement::NumberComparator;
    use crate::skip_list::level_generator::RandomLevelGenerator;
    use crate::skip_list::list::SkipList;
    use bytes::Bytes;
    use rand::random;
    use std::collections::HashSet;
    use std::mem::size_of;
    use std::ptr::slice_from_raw_parts;

    fn create_skip_list(max_level: usize) -> SkipList<NumberComparator<u32>> {
        let level_generator = RandomLevelGenerator::new(max_level, 0.5);

        return SkipList::new(1024 * 300, Box::new(level_generator));
    }

    pub fn get_bytes(n: u32) -> Bytes {
        let ptr = Box::into_raw(Box::new(n)) as *const u8;
        Bytes::copy_from_slice(unsafe {
            slice_from_raw_parts(ptr, size_of::<u32>())
                .as_ref()
                .unwrap()
        })
    }

    pub fn get_num(bytes: &Bytes) -> u32 {
        unsafe { *(bytes.as_ref().as_ptr() as *const u32) }
    }

    #[test]
    fn test_simple() {
        let mut skip_list = create_skip_list(3);

        skip_list.insert(get_bytes(3), get_bytes(3));
        skip_list.insert(get_bytes(5), get_bytes(5));
        skip_list.insert(get_bytes(6), get_bytes(6));
        skip_list.insert(get_bytes(1), get_bytes(1));

        let mut result_vec = vec![];
        let mut visitor = skip_list.visitor();
        while visitor.current_level() != 0 {
            visitor.reduce_level()
        }
        while visitor.peek_ref().is_some() {
            result_vec.push(get_num(visitor.peek_ref().unwrap().key()));
            visitor.next();
        }
        assert_eq!(vec![1, 3, 5, 6], result_vec);
    }

    #[test]
    fn random_test_insert() {
        let mut skip_list = create_skip_list(9);

        let mut set = HashSet::new();

        for _ in 0..100 {
            let key = loop {
                let result = random::<u32>();

                if !set.contains(&result) {
                    break result;
                }
            };

            skip_list.insert(get_bytes(key), get_bytes(key));
            set.insert(key);
        }

        let mut set_vec = set.iter().map(|key| key.clone()).collect::<Vec<u32>>();
        set_vec.sort();
        let mut result_vec = vec![];
        let mut visitor = skip_list.visitor();
        while visitor.current_level() != 0 {
            visitor.reduce_level()
        }
        while visitor.peek_ref().is_some() {
            result_vec.push(get_num(visitor.peek_ref().unwrap().key()));
            visitor.next();
        }
        assert_eq!(set_vec, result_vec);

        for key in set_vec {
            assert!(skip_list.seek(&get_bytes(key)).is_some());
        }

        for _ in 0..100 {
            let key = loop {
                let result = random::<u32>();

                if !set.contains(&result) {
                    break result;
                }
            };

            assert!(skip_list.seek(&get_bytes(key)).is_none());
        }
    }
}
