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
use std::sync::atomic::{AtomicPtr, Ordering as AtomicOrdering};

pub struct SkipList<C: Comparator> {
    entry: AtomicPtr<Node>,
    arena: Arena<Node>,
    len: AtomicUsize,
    level_generator: Box<dyn LevelGenerator>,
    height: AtomicUsize,
    _key_comparator: PhantomData<C>,
}

unsafe impl<C: Comparator> Sync for SkipList<C> {}

#[allow(dead_code)]
impl<C: Comparator> SkipList<C> {
    pub fn new(arena_capacity: u32, level_generator: Box<dyn LevelGenerator>) -> SkipList<C> {
        let arena = Arena::with_capacity(arena_capacity);

        let entry_node = Node::allocate_with_arena(
            Bytes::new(),
            Bytes::new(),
            level_generator.max_level(),
            &arena,
        );

        let entry = unsafe { arena.get_mut(entry_node) };

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
        let mut prev_next = self.find_position(&key);

        for i in prev_next.iter() {
            // duplicate key
            if i.0 == i.1 {
                return;
            }
        }

        let level = self.level_generator.generate_level();

        while prev_next.len() > level + 1 {
            prev_next.pop();
        }

        let entry_offset = self
            .arena
            .get_offset(self.entry.load(AtomicOrdering::SeqCst));

        while prev_next.len() < level + 1 {
            prev_next.push((entry_offset, 0));
        }

        let node_offset = Node::allocate_with_arena(key, value, level, &self.arena);
        let node = unsafe { &mut *self.arena.get_mut(node_offset) };

        for (level, (prev_offset, next_offset)) in prev_next.into_iter().enumerate().rev() {
            let mut prev = prev_offset;
            let mut next = next_offset;
            loop {
                let prev_node = unsafe { &mut *self.arena.get_mut(prev) };

                node.set_next(level, next);

                match prev_node.get_next_atomic(level).compare_exchange(
                    next,
                    node_offset,
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
            if height < level {
                match self.height.compare_exchange(
                    height,
                    level,
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
        let mut level = self.height();
        let prev = self
            .arena
            .get_offset(self.entry.load(AtomicOrdering::SeqCst));

        loop {
            let (prev, _) = self.find_position_for_level(prev, key, level);
            level -= 1;

            if level == 0 {
                break prev;
            }
        }
    }

    fn internal_visitor(&self) -> SkipListInternalVisitor<C> {
        SkipListInternalVisitor::create(
            unsafe { NonNull::new_unchecked(self.entry.load(AtomicOrdering::SeqCst)) },
            self.height(),
            &self.arena,
        )
    }

    fn find_position(&self, key: &Bytes) -> Vec<(u32, u32)> {
        let mut level = self.height();
        let mut result = vec![];
        let mut prev = self
            .arena
            .get_offset(self.entry.load(AtomicOrdering::SeqCst));
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

    fn find_position_for_level(&self, start_offset: u32, key: &Bytes, level: usize) -> (u32, u32) {
        let mut visitor = self.internal_visitor();
        visitor.set_offset(start_offset);
        assert!(visitor.current_ref().unwrap().height() >= level);

        visitor.set_level(level);

        loop {
            match visitor.compare_next_key(key) {
                Ordering::Less => {
                    visitor.next();
                }
                Ordering::Equal => {
                    break (visitor.peek_offset(), visitor.peek_offset());
                }
                _ => {
                    if let Ordering::Equal = visitor.compare_key(key) {
                        break (visitor.current_offset(), visitor.current_offset());
                    } else {
                        break (visitor.current_offset(), visitor.peek_offset());
                    };
                }
            }
        }
    }

    fn height(&self) -> usize {
        self.height.load(AtomicOrdering::SeqCst)
    }
}
