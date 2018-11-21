use std::time::{Duration, Instant};

pub trait TtlSet {
    fn insert(&mut self, item: u64, duration: Duration);
    // &mut because we want to permit cleanup operations
    fn contains(&mut self, item: u64) -> bool;
}

pub trait Clock: Default {
    // &mut for bad reasons, too lazy to make auto_advance threadsafe in FakeClock
    fn now(&mut self) -> Instant;
}

pub mod heap;
pub mod heap_cleanup;
pub mod mut_heap_cleanup;
pub mod redactor;
pub mod tree_cleanup;

#[cfg(test)]
pub mod fake_clock;
