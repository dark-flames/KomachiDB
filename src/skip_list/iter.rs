use crate::interface::Key;
use crate::skip_list::node::Node;

pub(super) struct SkipListInternalIterator<'a, K: Key> {
    current: Option<*mut Node<'a, K>>,
    current_level: usize,
}

#[allow(dead_code)]
impl<'a, K: Key> SkipListInternalIterator<'a, K> {
    pub fn new(entry: *mut Node<'a, K>, max_level: usize) -> SkipListInternalIterator<'a, K> {
        SkipListInternalIterator {
            current: Some(entry),
            current_level: max_level,
        }
    }

    pub fn current_as_ref(&self) -> Option<&Node<'a, K>> {
        unsafe { self.current().map(|ptr| &*ptr) }
    }

    pub fn current(&self) -> Option<*mut Node<'a, K>> {
        self.current
    }

    pub fn current_level(&self) -> usize {
        self.current_level
    }

    pub fn peek(&self) -> Option<*mut Node<'a, K>> {
        self.current
            .map(|ptr| unsafe { (*ptr).next.get(self.current_level).copied() })
            .flatten()
    }

    pub fn peek_as_ref(&self) -> Option<&Node<'a, K>> {
        unsafe { self.peek().map(|ptr| &*ptr) }
    }

    pub fn next(&mut self) -> Option<*mut Node<'a, K>> {
        let result = self.peek();

        if let Some(next_node) = result {
            self.current = Some(next_node)
        }

        result
    }

    pub fn next_level(&mut self) -> usize {
        if self.current_level > 0 {
            self.current_level -= 1
        }

        self.current_level
    }
}
