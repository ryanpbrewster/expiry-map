use std::collections::HashMap;
use std::time::{Duration, Instant};

use {Clock, TtlSet};
/// A simple TtlSet that keeps track of each item's expiration time.
/// During a `contains` check, it inspects the expiration time; if it is expired, returns `false`.
pub struct Redactor<C: Clock> {
    clock: C,
    expiration_times: HashMap<u64, Instant>,
}

impl<C: Clock> Redactor<C> {
    pub fn new() -> Redactor<C> {
        Redactor {
            clock: C::new(),
            expiration_times: HashMap::new(),
        }
    }
}

impl<C: Clock> TtlSet for Redactor<C> {
    fn insert(&mut self, item: u64, duration: Duration) {
        self.expiration_times
            .insert(item, self.clock.now() + duration);
    }

    fn contains(&mut self, key: u64) -> bool {
        match self.expiration_times.get(&key) {
            Some(expires_at) => self.clock.now() < *expires_at,
            None => false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use fake_clock::FakeClock;

    #[test]
    fn smoke_test() {
        let mut m = Redactor::<FakeClock>::new();

        assert!(!m.contains(0));

        m.insert(0, Duration::from_secs(15));
        assert!(m.contains(0));

        m.clock.advance(Duration::from_secs(10));
        assert!(m.contains(0));

        m.clock.advance(Duration::from_secs(10));
        assert!(!m.contains(0));
    }
}
