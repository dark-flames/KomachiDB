use crate::skip_list::arena::Arena;
use crate::skip_list::comparator::Comparator;
use crate::skip_list::node::Node;
use crate::skip_list::SkipList;
use bytes::Bytes;
use std::cmp::{max, Ordering};
use std::marker::PhantomData;
use std::ptr::NonNull;

#[derive(Clone)]
pub struct SkipListInternalVisitor<'a, C: Comparator> {
    arena_ref: &'a Arena,
    current: NonNull<Node>,
    level: usize,
    valid: bool,
    _key_comparator: PhantomData<C>,
}

#[allow(dead_code)]
impl<'a, C: Comparator> SkipListInternalVisitor<'a, C> {
    pub fn create(entry: NonNull<Node>, level: usize, arena_ref: &'a Arena) -> Self {
        SkipListInternalVisitor {
            arena_ref,
            current: entry,
            level,
            valid: true,
            _key_comparator: PhantomData::default(),
        }
    }

    pub fn set_offset(&mut self, offset: u32) {
        if offset != 0 {
            self.current = unsafe { NonNull::new_unchecked(self.arena_ref.get_mut_node(offset)) };
            self.level = self.current_ref().unwrap().height();
        } else {
            self.valid = true;
        };
    }

    pub fn set_level(&mut self, level: usize) {
        self.level = level
    }

    pub fn set_zero_level(&mut self) {
        self.level = 0;
    }

    pub fn is_head(&self) -> bool {
        self.current_ref()
            .map(|node| node.is_head())
            .unwrap_or(false)
    }

    pub fn valid(&self) -> bool {
        self.valid
    }

    pub fn current_offset(&self) -> u32 {
        if self.valid {
            self.arena_ref.get_offset(self.current.as_ptr())
        } else {
            0
        }
    }

    pub fn current_ref(&self) -> Option<&Node> {
        if self.valid {
            Some(unsafe { self.current.as_ref() })
        } else {
            None
        }
    }

    pub fn peek_offset(&self) -> u32 {
        let offset_option = self.current_ref().map(|node| node.next_offset(self.level));

        match offset_option {
            Some(offset) if offset != 0 => offset,
            _ => 0,
        }
    }

    pub fn peek_ref(&self) -> Option<&'a Node> {
        match self.peek_offset() {
            0 => None,
            offset => Some(unsafe { &*self.arena_ref.get_node(offset) }),
        }
    }

    pub fn next(&mut self) -> u32 {
        let offset = self.peek_offset();
        if offset != 0 {
            self.current = unsafe { NonNull::new_unchecked(self.arena_ref.get_mut_node(offset)) };
        } else {
            self.valid = false;
        };

        offset
    }

    pub fn current_level(&self) -> usize {
        self.level
    }

    pub fn reduce_level(&mut self) {
        self.level = max(self.level - 1, 0)
    }

    pub fn key(&self) -> Option<&Bytes> {
        self.current_ref().map(|node| node.key()).flatten()
    }

    pub fn value(&self) -> Option<&Bytes> {
        self.current_ref().map(|node| node.value()).flatten()
    }

    pub fn compare_key(&self, key: &Bytes) -> Ordering {
        if self.current_ref().is_none() {
            Ordering::Greater
        } else if self.is_head() {
            Ordering::Less
        } else {
            C::compare(self.key().unwrap().as_ref(), key.as_ref())
        }
    }

    pub fn compare_next_key(&self, key: &Bytes) -> Ordering {
        if let Some(next) = self.peek_ref() {
            C::compare(next.key().unwrap().as_ref(), key.as_ref())
        } else {
            Ordering::Greater
        }
    }

    pub fn compare_and_get_next_offset(&self, key: &Bytes) -> (Ordering, u32) {
        let next_offset = self.peek_offset();
        if next_offset != 0 {
            let next = unsafe { &*self.arena_ref.get_node(next_offset) };
            (
                C::compare(next.key().unwrap().as_ref(), key.as_ref()),
                next_offset,
            )
        } else {
            (Ordering::Greater, 0)
        }
    }

    pub fn seek(&mut self, key: &Bytes) {
        let offset = loop {
            match self.compare_and_get_next_offset(key) {
                (Ordering::Less, offset) => {
                    let level = self.current_level();
                    if offset != 0 {
                        self.set_offset(offset);
                        self.set_level(level)
                    }
                }
                (Ordering::Equal, offset) => {
                    break offset;
                }
                (Ordering::Greater, _) if self.current_level() == 0 => {
                    break 0;
                }
                (Ordering::Greater, _) => {
                    self.reduce_level();
                }
            }
        };

        self.set_offset(offset);
        self.set_zero_level();
    }
}

pub struct SkipListIterator<'a, C: Comparator> {
    internal_visitor: SkipListInternalVisitor<'a, C>,
}

impl<'a, C: Comparator> From<SkipListInternalVisitor<'a, C>> for SkipListIterator<'a, C> {
    fn from(mut visitor: SkipListInternalVisitor<'a, C>) -> Self {
        visitor.set_zero_level();

        SkipListIterator {
            internal_visitor: visitor,
        }
    }
}

impl<'a, C: Comparator> Iterator for SkipListIterator<'a, C> {
    type Item = (&'a Bytes, &'a Bytes);

    fn next(&mut self) -> Option<Self::Item> {
        let node_option = self.internal_visitor.peek_ref();

        self.internal_visitor.next();

        node_option
            .map(|node| {
                if !node.is_head() {
                    Some((node.key().unwrap(), node.value().unwrap()))
                } else {
                    None
                }
            })
            .flatten()
    }
}

#[derive(Clone)]
pub struct SkipListVisitor<'a, C: Comparator> {
    skip_list: &'a SkipList<C>,
    internal_visitor: SkipListInternalVisitor<'a, C>,
}

#[allow(dead_code)]
impl<'a, C: Comparator> SkipListVisitor<'a, C> {
    pub fn new(
        skip_list: &'a SkipList<C>,
        internal_visitor: SkipListInternalVisitor<'a, C>,
    ) -> Self {
        SkipListVisitor {
            skip_list,
            internal_visitor,
        }
    }
    pub fn key(&self) -> Option<&Bytes> {
        self.internal_visitor.key()
    }

    pub fn value(&self) -> Option<&Bytes> {
        self.internal_visitor.value()
    }

    pub fn next(&mut self) {
        assert!(self.valid());
        self.internal_visitor.next();
    }

    pub fn seek(&mut self, key: &Bytes) {
        self.internal_visitor.seek(key);
    }

    pub fn valid(&self) -> bool {
        self.internal_visitor.valid
    }

    fn set_offset(&mut self, offset: u32) {
        self.internal_visitor.set_offset(offset);
        self.internal_visitor.set_zero_level()
    }
}
