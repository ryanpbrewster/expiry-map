use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use heap::{Handle, MutHeap};
use {Clock, TtlSet};

#[derive(Default)]
pub struct HeapCleanup<C: Clock> {
    clock: C,
    expiration_index: MutHeap<Expiration>,
    expiration_times: HashMap<u64, Handle>,
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
            match self.expiration_index.peek_max() {
                Some(exp) if exp.time <= threshold => self.expiration_times.remove(&exp.item),
                _ => break,
            };
            self.expiration_index.pop_max();
        }
    }
}

impl<C: Clock> TtlSet for HeapCleanup<C> {
    fn insert(&mut self, item: u64, duration: Duration) {
        let time = self.clock.now() + duration;
        match self.expiration_times.entry(item) {
            Entry::Occupied(occ) => {
                self.expiration_index.decrement(occ.get(), |x| {
                    if time < x.time {
                        x.time = time;
                    }
                });
                self.expiration_index.increment(occ.get(), |x| {
                    if time > x.time {
                        x.time = time;
                    }
                });
            }
            Entry::Vacant(vac) => {
                let handle = self.expiration_index.insert(Expiration { item, time });
                vac.insert(handle);
            }
        }
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
