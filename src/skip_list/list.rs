use crate::skip_list::arena::Arena;
use crate::skip_list::comparator::Comparator;
use crate::skip_list::iter::{SkipListInternalVisitor, SkipListIterator, SkipListVisitor};
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
        let mut prev = self.find_position(&key);

        let node_offset = Node::allocate_with_arena(key, value, level, &mut self.arena);
        let node = unsafe { &mut *self.arena.get_mut(node_offset) };

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

    pub fn len(&self) -> usize {
        self.len.load(AtomicOrdering::SeqCst)
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

    pub fn seek_offset(&self, key: &Bytes) -> u32 {
        let mut internal_visitor = self.internal_visitor();

        loop {
            match internal_visitor.compare_next_key(key) {
                Ordering::Less => {
                    internal_visitor.next();
                }
                Ordering::Equal => {
                    break internal_visitor.peek_offset();
                }
                Ordering::Greater if internal_visitor.current_level() == 0 => {
                    break 0;
                }
                Ordering::Greater => {
                    internal_visitor.reduce_level();
                }
            }
        }
    }

    pub fn seek_prev_offset(&self, key: &Bytes) -> u32 {
        let prev = self.find_position(key);
        let mut seek_visitor = self.internal_visitor();
        seek_visitor.set_offset(prev[0]);
        seek_visitor.set_zero_level();
        if seek_visitor.compare_next_key(key) == Ordering::Equal {
            prev[0]
        } else {
            0
        }
    }

    fn internal_visitor(&self) -> SkipListInternalVisitor<C> {
        SkipListInternalVisitor::create(self.entry, self.height(), &self.arena)
    }

    fn find_position(&self, key: &Bytes) -> Vec<u32> {
        let mut internal_visitor = self.internal_visitor();
        let mut prev = vec![];

        loop {
            if let Ordering::Less = internal_visitor.compare_next_key(key) {
                internal_visitor.next();
            } else {
                if let Ordering::Less = internal_visitor.compare_key(key) {
                    prev.push(internal_visitor.current_offset())
                }

                if internal_visitor.current_level() == 0 {
                    break;
                } else {
                    internal_visitor.reduce_level()
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
