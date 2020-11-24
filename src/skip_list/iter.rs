use crate::skip_list::node::Node;
use crate::skip_list::SkipList;
use crate::Comparator;
use bytes::Bytes;
use std::cmp::{max, Ordering};
use std::marker::PhantomData;
use std::ptr::{null_mut, NonNull};

#[derive(Clone)]
pub struct SkipListInternalVisitor<'a, C: Comparator> {
    skip_list: &'a SkipList<C>,
    current: NonNull<Node>,
    level: usize,
    valid: bool,
    _key_comparator: PhantomData<C>,
}

#[allow(dead_code)]
impl<'a, C: Comparator> SkipListInternalVisitor<'a, C> {
    pub fn create(entry: NonNull<Node>, level: usize, list: &'a SkipList<C>) -> Self {
        SkipListInternalVisitor {
            skip_list: list,
            current: entry,
            level,
            valid: true,
            _key_comparator: PhantomData::default(),
        }
    }

    pub fn set_current(&mut self, current: *mut Node) {
        match NonNull::new(current) {
            Some(ptr) => {
                self.current = ptr;
                self.level = self.current_ref().unwrap().height();
            }
            _ => {
                self.valid = false;
            }
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

    pub fn current_ptr(&self) -> Option<NonNull<Node>> {
        if self.valid {
            Some(self.current)
        } else {
            None
        }
    }

    pub fn current_ref(&self) -> Option<&'a Node> {
        self.current_ptr()
            .map(|ptr| unsafe { ptr.as_ptr().as_ref().unwrap() })
    }

    fn peek_ptr(&self) -> Option<*mut Node> {
        self.current_ref().map(|node| node.next(self.level))
    }

    pub fn peek(&self) -> Option<&'a mut Node> {
        self.peek_ptr().map(|ptr| unsafe { ptr.as_mut() }).flatten()
    }

    pub fn next(&mut self) -> Option<&'a mut Node> {
        let result = self.peek();

        match self.peek_ptr() {
            Some(node) => match NonNull::new(node) {
                Some(ptr) => {
                    self.current = ptr;
                }
                _ => {
                    self.valid = false;
                }
            },
            _ => {
                self.valid = false;
            }
        }

        result
    }

    pub fn current_level(&self) -> usize {
        self.level
    }

    pub fn reduce_level(&mut self) {
        self.level = max(self.level - 1, 0)
    }

    pub fn key(&self) -> Option<&'a [u8]> {
        self.current_ref().map(|key| key.key()).flatten()
    }

    pub fn value(&self) -> Option<&'a [u8]> {
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
        if let Some(next) = self.peek() {
            C::compare(next.key().unwrap().as_ref(), key.as_ref())
        } else {
            Ordering::Greater
        }
    }

    pub fn compare_and_get_next(&self, key: &[u8]) -> (Ordering, Option<*mut Node>) {
        match self.peek() {
            Some(next) => (C::compare(next.key().unwrap(), key), Some(next)),
            _ => (Ordering::Greater, None),
        }
    }

    pub fn seek(&mut self, key: &[u8], less_or_equal: bool) {
        assert!(self
            .current_ref()
            .map(|node| node.is_head())
            .unwrap_or(false));

        let result = loop {
            match self.compare_and_get_next(key) {
                (Ordering::Less, next) => {
                    let level = self.current_level();
                    if let Some(next_ptr) = next {
                        self.set_current(next_ptr);
                        self.set_level(level)
                    }
                }
                (Ordering::Equal, next) => {
                    break next.unwrap();
                }
                (Ordering::Greater, _) if self.current_level() == 0 => {
                    break if less_or_equal {
                        self.current.as_ptr()
                    } else {
                        null_mut()
                    }
                }
                (Ordering::Greater, _) => {
                    self.reduce_level();
                }
            };
        };

        self.set_current(result);
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
    type Item = (&'a [u8], &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        let node_option = self.internal_visitor.peek();

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
    pub fn key(&self) -> Option<&'a [u8]> {
        self.internal_visitor.key()
    }

    pub fn value(&self) -> Option<&'a [u8]> {
        self.internal_visitor.value()
    }

    pub fn next(&mut self) {
        assert!(self.valid());
        self.internal_visitor.next();
    }

    pub fn seek(&mut self, key: &[u8]) {
        self.internal_visitor.seek(key, false);
    }

    pub fn seek_less_or_equal(&mut self, key: &[u8]) {
        self.internal_visitor.seek(key, true)
    }

    pub fn valid(&self) -> bool {
        self.internal_visitor.valid
    }

    fn set_current(&mut self, current: *mut Node) {
        self.internal_visitor.set_current(current);
    }
}
