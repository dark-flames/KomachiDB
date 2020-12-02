mod chunk;
mod log_iterator;
mod log_manager;
mod record;
#[cfg(test)]
mod tests;

pub use log_manager::{LogManager, LogNumber};
