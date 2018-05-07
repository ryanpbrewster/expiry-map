#![allow(dead_code)]

use std::collections::{BTreeMap, HashMap, HashSet};
use std::mem;
use std::time::{Duration, Instant};

pub trait ExpMap<'a> {
    fn get<'b>(&'a mut self, key: u32) -> Option<&'b str>
    where
        'a: 'b;
    fn put(&mut self, key: u32, value: String, duration: Duration);
}

pub trait Clock {
    fn now(&self) -> Instant;
}

pub struct FakeClock(Instant);
impl Clock for FakeClock {
    fn now(&self) -> Instant {
        self.0
    }
}
impl FakeClock {
    fn advance(&mut self, duration: Duration) {
        self.0 += duration;
    }
}

pub struct MonotonicImpl<C: Clock> {
    key_map: HashMap<u32, Wrapper>,
    clock: C,
}
struct Wrapper {
    value: String,
    expiry: Instant,
}

impl MonotonicImpl<FakeClock> {
    fn new() -> MonotonicImpl<FakeClock> {
        MonotonicImpl {
            key_map: HashMap::new(),
            clock: FakeClock(Instant::now()),
        }
    }
}

impl<'a, C: Clock> ExpMap<'a> for MonotonicImpl<C> {
    fn get<'b>(&'a mut self, key: u32) -> Option<&'b str>
    where
        'a: 'b,
    {
        let wrap = self.key_map.get(&key)?;
        if self.clock.now() < wrap.expiry {
            Some(&wrap.value)
        } else {
            None
        }
    }

    fn put(&mut self, key: u32, value: String, duration: Duration) {
        self.key_map.insert(
            key,
            Wrapper {
                value,
                expiry: self.clock.now() + duration,
            },
        );
    }
}

pub struct IncrementalImpl<C: Clock> {
    key_map: HashMap<u32, Wrapper>,
    expiry_map: BTreeMap<Instant, HashSet<u32>>,
    clock: C,
}
impl IncrementalImpl<FakeClock> {
    fn new() -> IncrementalImpl<FakeClock> {
        IncrementalImpl {
            key_map: HashMap::new(),
            expiry_map: BTreeMap::new(),
            clock: FakeClock(Instant::now()),
        }
    }
}

impl<C: Clock> IncrementalImpl<C> {
    fn incremental_clean(&mut self, threshold: Instant) {
        let mut tmp = self.expiry_map.split_off(&threshold);
        mem::swap(&mut self.expiry_map, &mut tmp);
        for (_expiry, ids) in tmp {
            for id in ids {
                self.key_map.remove(&id);
            }
        }
    }
}

impl<'a, C: Clock> ExpMap<'a> for IncrementalImpl<C> {
    fn get<'b>(&'a mut self, key: u32) -> Option<&'b str>
    where
        'a: 'b,
    {
        let now = self.clock.now();
        self.incremental_clean(now);
        let wrapper = self.key_map.get(&key)?;
        Some(&wrapper.value)
    }

    fn put(&mut self, key: u32, value: String, duration: Duration) {
        let expiry = self.clock.now() + duration;
        if let Some(prev) = self.key_map.insert(key, Wrapper { value, expiry }) {
            let size_after_deleting = {
                let ids_to_expire = self.expiry_map
                    .get_mut(&prev.expiry)
                    .expect("the previous entry must have had an expiration time registered");
                ids_to_expire.remove(&key);
                ids_to_expire.len()
            };
            if size_after_deleting == 0 {
                self.expiry_map.remove(&prev.expiry);
            }
        }
        self.expiry_map
            .entry(expiry)
            .or_insert_with(HashSet::new)
            .insert(key);
    }
}

mod test {
    use super::*;

    #[test]
    fn smoke_monotonic() {
        let mut m = MonotonicImpl::new();

        assert_eq!(m.get(0), None);

        m.put(0, String::from("foo"), Duration::from_secs(15));
        assert_eq!(m.get(0), Some("foo"));

        m.clock.advance(Duration::from_secs(10));
        assert_eq!(m.get(0), Some("foo"));

        m.clock.advance(Duration::from_secs(10));
        assert_eq!(m.get(0), None);
    }

    #[test]
    fn smoke_incremental() {
        let mut m = IncrementalImpl::new();

        assert_eq!(m.get(0), None);

        m.put(0, String::from("foo"), Duration::from_secs(15));
        assert_eq!(m.get(0), Some("foo"));

        m.clock.advance(Duration::from_secs(10));
        assert_eq!(m.get(0), Some("foo"));

        m.clock.advance(Duration::from_secs(10));
        assert_eq!(m.get(0), None);
    }

    #[test]
    fn overwriting_entries_wiped_old_expirations() {
        let mut m = IncrementalImpl::new();

        assert_eq!(m.get(0), None);

        m.put(0, String::from("foo"), Duration::from_secs(15));
        assert_eq!(m.get(0), Some("foo"));

        m.put(0, String::from("bar"), Duration::from_secs(150));
        assert_eq!(m.get(0), Some("bar"));

        m.clock.advance(Duration::from_secs(100));
        assert_eq!(m.get(0), Some("bar"));
    }
}
