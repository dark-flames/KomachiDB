use crate::skip_list::arena::Arena;
use crate::skip_list::comparator::Comparator;
use crate::skip_list::iter::{SkipListInternalVisitor, SkipListIterator, SkipListVisitor};
use crate::skip_list::level_generator::LevelGenerator;
use crate::skip_list::node::Node;
use bytes::Bytes;
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::ptr::{null_mut, NonNull};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::{AtomicPtr, Ordering as AtomicOrdering};

pub struct SkipList<C: Comparator> {
    entry: AtomicPtr<Node>,
    arena: Arena,
    len: AtomicUsize,
    level_generator: Box<dyn LevelGenerator>,
    height: AtomicUsize,
    _key_comparator: PhantomData<C>,
}

unsafe impl<C: Comparator> Sync for SkipList<C> {}

#[allow(dead_code)]
impl<C: Comparator> SkipList<C> {
    pub fn new(level_generator: Box<dyn LevelGenerator>, block_size: usize) -> SkipList<C> {
        let arena = Arena::new(block_size);

        let entry = Node::head(level_generator.max_level(), &arena);

        SkipList {
            entry: AtomicPtr::new(entry),
            arena,
            len: AtomicUsize::new(0),
            level_generator,
            height: AtomicUsize::new(0),
            _key_comparator: Default::default(),
        }
    }

    pub fn insert(&self, key: Bytes, value: Bytes) {
        let mut prev_next = self.find_position(key.as_ref());

        for i in prev_next.iter() {
            // duplicate key
            if i.0 == i.1 {
                return;
            }
        }

        let node_level = self.level_generator.generate_level();

        while prev_next.len() > node_level + 1 {
            prev_next.pop();
        }

        let entry = self.entry.load(AtomicOrdering::SeqCst);

        while prev_next.len() < node_level + 1 {
            prev_next.push((entry, null_mut()));
        }

        let node_ptr = Node::allocate_with_arena(key, value, node_level, &self.arena);

        let node = unsafe { node_ptr.as_mut().unwrap() };

        for (level, (mut prev, mut next)) in prev_next.into_iter().enumerate() {
            loop {
                let prev_node = unsafe { prev.as_mut().unwrap() };

                node.set_next(level, next);

                match prev_node.get_next_atomic(level).compare_exchange(
                    next,
                    node_ptr,
                    AtomicOrdering::SeqCst,
                    AtomicOrdering::SeqCst,
                ) {
                    Ok(_) => {
                        break;
                    }
                    Err(_) => {
                        let result = self.find_position_for_level(prev, node.key().unwrap(), level);
                        prev = result.0;
                        next = result.1;

                        if prev == next {
                            return;
                        }
                    }
                }
            }
        }
        let mut height = self.height();
        loop {
            if height < node_level {
                match self.height.compare_exchange(
                    height,
                    node_level,
                    AtomicOrdering::SeqCst,
                    AtomicOrdering::SeqCst,
                ) {
                    Ok(_) => {
                        break;
                    }
                    Err(h) => {
                        height = h;
                    }
                }
            } else {
                break;
            }
        }
        self.len.fetch_add(1, AtomicOrdering::SeqCst);
    }

    pub fn len(&self) -> usize {
        self.len.load(AtomicOrdering::SeqCst)
    }

    pub fn memory_usage(&self) -> usize {
        self.arena.memory_usage()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> SkipListIterator<C> {
        SkipListIterator::from(self.internal_visitor())
    }

    pub fn visitor(&self) -> SkipListVisitor<C> {
        SkipListVisitor::new(self, self.internal_visitor())
    }

    fn internal_visitor(&self) -> SkipListInternalVisitor<C> {
        SkipListInternalVisitor::create(
            unsafe { NonNull::new_unchecked(self.entry.load(AtomicOrdering::SeqCst)) },
            self.height(),
            self,
        )
    }

    fn find_position(&self, key: &[u8]) -> Vec<(*mut Node, *mut Node)> {
        let mut level = self.height();
        let mut result = vec![];
        let mut prev = self.entry.load(AtomicOrdering::SeqCst);

        loop {
            let item = self.find_position_for_level(prev, key, level);
            prev = item.0;

            result.push(item);

            if level == 0 {
                break;
            } else {
                level -= 1;
            }
        }

        result.reverse();

        result
    }

    fn find_position_for_level(
        &self,
        start: *mut Node,
        key: &[u8],
        level: usize,
    ) -> (*mut Node, *mut Node) {
        let mut visitor = self.internal_visitor();
        visitor.set_current(start);
        assert!(visitor.current_ref().unwrap().height() >= level);

        visitor.set_level(level);

        loop {
            match visitor.compare_and_get_next(key) {
                (Ordering::Less, _) => {
                    visitor.next();
                }
                (Ordering::Equal, next) => {
                    break (next.unwrap(), next.unwrap());
                }
                (_, next) => {
                    break (
                        visitor.current_ptr().unwrap().as_ptr(),
                        next.unwrap_or(null_mut()),
                    );
                }
            }
        }
    }

    fn height(&self) -> usize {
        self.height.load(AtomicOrdering::SeqCst)
    }
}
