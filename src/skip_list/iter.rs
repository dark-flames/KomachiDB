use crate::interface::Key;
use crate::skip_list::node::Node;

pub struct SkipListVisitor<K: Key> {
    current: Option<*mut Node<K>>,
    current_level: usize,
}

#[allow(dead_code)]
impl<K: Key> SkipListVisitor<K> {
    pub fn new(entry: *mut Node<K>, max_level: usize) -> SkipListVisitor<K> {
        SkipListVisitor {
            current: Some(entry),
            current_level: max_level,
        }
    }

    pub fn current_as_ref(&self) -> Option<&Node<K>> {
        unsafe { self.current().map(|ptr| &*ptr) }
    }

    pub fn current(&self) -> Option<*mut Node<K>> {
        self.current
    }

    pub fn current_level(&self) -> usize {
        self.current_level
    }

    pub fn peek(&self) -> Option<*mut Node<K>> {
        self.current
            .map(|ptr| unsafe { (*ptr).next.get(self.current_level).copied() })
            .flatten()
    }

    pub fn peek_as_ref(&self) -> Option<&Node<K>> {
        unsafe { self.peek().map(|ptr| &*ptr) }
    }

    pub fn next(&mut self) -> Option<*mut Node<K>> {
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

impl<K: Key> Into<SkipListIterator<K>> for SkipListVisitor<K> {
    fn into(mut self) -> SkipListIterator<K> {
        self.current_level = 0;

        SkipListIterator::new(self)
    }
}

pub struct SkipListIterator<K: Key + 'static> {
    visitor: SkipListVisitor<K>,
}

impl<K: Key + 'static> SkipListIterator<K> {
    pub fn new(visitor: SkipListVisitor<K>) -> SkipListIterator<K> {
        SkipListIterator { visitor }
    }
}

impl<K: Key + 'static> Iterator for SkipListIterator<K> {
    type Item = (K, *const [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        let result = self
            .visitor
            .peek_as_ref()
            .map(|node| (*node.key_ref().unwrap(), node.ptr.unwrap()));

        self.visitor.next();

        result
    }
}
