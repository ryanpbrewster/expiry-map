use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::time::{Duration, Instant};

use {Clock, TtlSet};

#[derive(Default)]
pub struct HeapCleanup<C: Clock> {
    clock: C,
    expiration_times: HashMap<u64, Instant>,
    expiration_index: BinaryHeap<Expiration>,
}

struct Expiration {
    time: Instant,
    item: u64,
}
impl Ord for Expiration {
    // Larger element is the one that expires first, so that a max-heap will pop old elements
    fn cmp(&self, other: &Self) -> Ordering {
        self.time.cmp(&other.time).reverse()
    }
}
impl PartialOrd for Expiration {
    fn partial_cmp(&self, other: &Expiration) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for Expiration {
    fn eq(&self, other: &Expiration) -> bool {
        self.time == other.time
    }
}
impl Eq for Expiration {}

impl<C: Clock> HeapCleanup<C> {
    fn incremental_clean(&mut self, threshold: Instant) {
        loop {
            match self.expiration_index.peek() {
                Some(exp) if exp.time <= threshold => {
                    if let ::std::collections::hash_map::Entry::Occupied(occ) =
                        self.expiration_times.entry(exp.item)
                    {
                        if *occ.get() < threshold {
                            occ.remove();
                        }
                    }
                }
                _ => break,
            }
            self.expiration_index.pop();
        }
    }
}

impl<C: Clock> TtlSet for HeapCleanup<C> {
    fn insert(&mut self, item: u64, duration: Duration) {
        let time = self.clock.now() + duration;
        self.expiration_times.insert(item, time);
        self.expiration_index.push(Expiration { item, time });
    }

    fn contains(&mut self, item: u64) -> bool {
        let now = self.clock.now();
        self.incremental_clean(now);
        self.expiration_times.contains_key(&item)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use fake_clock::FakeClock;

    #[test]
    fn tree_cleanup_smoke_test() {
        let mut m = HeapCleanup::<FakeClock>::default();

        assert!(!m.contains(0));

        m.insert(0, Duration::from_secs(15));
        assert!(m.contains(0));

        m.clock.advance(Duration::from_secs(10));
        assert!(m.contains(0));

        m.clock.advance(Duration::from_secs(10));
        assert!(!m.contains(0));
    }

    #[test]
    fn overwriting_entries_wiped_old_expirations() {
        let mut m = HeapCleanup::<FakeClock>::default();

        assert!(!m.contains(0));

        m.insert(0, Duration::from_secs(15));
        assert!(m.contains(0));

        m.insert(0, Duration::from_secs(150));
        assert!(m.contains(0));

        m.clock.advance(Duration::from_secs(100));
        assert!(m.contains(0));
    }
}
