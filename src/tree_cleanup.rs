use std::collections::{BTreeMap, HashMap, HashSet};
use std::mem;
use std::time::{Duration, Instant};

use {Clock, TtlSet};

pub struct TreeCleanup<C: Clock> {
    clock: C,
    expiration_times: HashMap<u64, Instant>,
    expiration_index: BTreeMap<Instant, HashSet<u64>>,
}
impl<C: Clock> TreeCleanup<C> {
    pub fn new() -> TreeCleanup<C> {
        TreeCleanup {
            clock: C::new(),
            expiration_times: HashMap::new(),
            expiration_index: BTreeMap::new(),
        }
    }
}

impl<C: Clock> TreeCleanup<C> {
    fn incremental_clean(&mut self, threshold: Instant) {
        let mut tmp = self.expiration_index.split_off(&threshold);
        mem::swap(&mut self.expiration_index, &mut tmp);
        for (_expiry, ids) in tmp {
            for id in ids {
                self.expiration_times.remove(&id);
            }
        }
    }
}

impl<C: Clock> TtlSet for TreeCleanup<C> {
    fn insert(&mut self, item: u64, duration: Duration) {
        let expiry = self.clock.now() + duration;
        if let Some(prev) = self.expiration_times.insert(item, expiry) {
            let size_after_deleting = {
                let ids_to_expire = self
                    .expiration_index
                    .get_mut(&prev)
                    .expect("the previous entry must have had an expiration time registered");
                ids_to_expire.remove(&item);
                ids_to_expire.len()
            };
            if size_after_deleting == 0 {
                self.expiration_index.remove(&prev);
            }
        }
        self.expiration_index
            .entry(expiry)
            .or_insert_with(HashSet::new)
            .insert(item);
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
        let mut m = TreeCleanup::<FakeClock>::new();

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
        let mut m = TreeCleanup::<FakeClock>::new();

        assert!(!m.contains(0));

        m.insert(0, Duration::from_secs(15));
        assert!(m.contains(0));

        m.insert(0, Duration::from_secs(150));
        assert!(m.contains(0));

        m.clock.advance(Duration::from_secs(100));
        assert!(m.contains(0));
    }
}
