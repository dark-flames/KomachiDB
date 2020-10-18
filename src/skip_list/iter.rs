use crate::skip_list::arena::Arena;
use crate::skip_list::comparator::Comparator;
use crate::skip_list::node::Node;
use bytes::Bytes;
use std::cmp::{max, Ordering};
use std::marker::PhantomData;
use std::ptr::NonNull;

pub struct SkipListVisitor<'a, C: Comparator> {
    arena_ref: &'a Arena<Node>,
    current: NonNull<Node>,
    level: usize,
    _key_comparator: PhantomData<C>,
}

impl<'a, C: Comparator> SkipListVisitor<'a, C> {
    pub fn create(entry: NonNull<Node>, level: usize, arena_ref: &'a Arena<Node>) -> Self {
        SkipListVisitor {
            arena_ref,
            current: entry,
            level,
            _key_comparator: PhantomData::default(),
        }
    }
    pub fn current_offset(&self) -> u32 {
        self.arena_ref.get_offset(self.current.as_ptr())
    }

    pub fn current_ref(&self) -> &Node {
        unsafe { self.current.as_ref() }
    }

    pub fn peek_offset(&self) -> Option<u32> {
        let offset = self.current_ref().next_offset(self.level);

        if offset == 0 {
            None
        } else {
            Some(offset)
        }
    }

    pub fn peek_ref(&self) -> Option<&Node> {
        self.peek_offset()
            .map(|offset| unsafe { &*self.arena_ref.get(offset) })
    }

    pub fn next(&mut self) -> Option<u32> {
        if let Some(offset) = self.peek_offset() {
            self.current = unsafe { NonNull::new_unchecked(self.arena_ref.get_mut(offset)) };
            Some(offset)
        } else {
            None
        }
    }

    pub fn current_level(&self) -> usize {
        self.level
    }

    pub fn reduce_level(&mut self) {
        self.level = max(self.level - 1, 0)
    }

    pub fn compare_key(&self, key: &Bytes) -> Ordering {
        if self.current_ref().is_head() {
            Ordering::Less
        } else {
            C::compare(self.current_ref().key().as_ref(), key.as_ref())
        }
    }

    pub fn compare_next_key(&self, key: &Bytes) -> Ordering {
        if let Some(next) = self.peek_ref() {
            C::compare(next.key().as_ref(), key.as_ref())
        } else {
            Ordering::Greater
        }
    }
}
