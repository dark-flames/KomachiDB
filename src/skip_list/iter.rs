use crate::comparator::Comparator;
use crate::skip_list::node::Node;

pub struct SkipListVisitor<C: Comparator> {
    current: Option<*mut Node<C>>,
    current_level: usize,
}

#[allow(dead_code)]
impl<C: Comparator> SkipListVisitor<C> {
    pub fn new(entry: *mut Node<C>, max_level: usize) -> SkipListVisitor<C> {
        SkipListVisitor {
            current: Some(entry),
            current_level: max_level,
        }
    }

    pub fn current_as_ref(&self) -> Option<&Node<C>> {
        unsafe { self.current().map(|ptr| &*ptr) }
    }

    pub fn current(&self) -> Option<*mut Node<C>> {
        self.current
    }

    pub fn current_level(&self) -> usize {
        self.current_level
    }

    pub fn peek(&self) -> Option<*mut Node<C>> {
        self.current
            .map(|ptr| unsafe { (*ptr).next.get(self.current_level).copied() })
            .flatten()
    }

    pub fn peek_as_ref(&self) -> Option<&Node<C>> {
        unsafe { self.peek().map(|ptr| &*ptr) }
    }

    pub fn next(&mut self) -> Option<*mut Node<C>> {
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

impl<C: Comparator> Into<SkipListIterator<C>> for SkipListVisitor<C> {
    fn into(mut self) -> SkipListIterator<C> {
        self.current_level = 0;

        SkipListIterator::new(self)
    }
}

pub struct SkipListIterator<C: Comparator> {
    visitor: SkipListVisitor<C>,
}

impl<C: Comparator> SkipListIterator<C> {
    pub fn new(visitor: SkipListVisitor<C>) -> SkipListIterator<C> {
        SkipListIterator { visitor }
    }
}

impl<C: Comparator> Iterator for SkipListIterator<C> {
    type Item = (*const [u8], *const u8);

    fn next(&mut self) -> Option<Self::Item> {
        let result = self
            .visitor
            .peek_as_ref()
            .map(|node| (node.key.unwrap(), node.ptr.unwrap()));

        self.visitor.next();

        result
    }
}
