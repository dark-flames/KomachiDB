mod arena;
pub mod comparator;
mod iter;
mod level_generator;
mod list;
mod node;
#[cfg(test)]
mod tests;

pub use comparator::{Comparator, NumberComparator};
pub use iter::{SkipListIterator, SkipListVisitor};
pub use level_generator::{LevelGenerator, RandomLevelGenerator};
pub use list::SkipList;

const MAX_HEIGHT: usize = 20;
