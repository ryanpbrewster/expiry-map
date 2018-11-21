use std::time::{Duration, Instant};
use Clock;

pub struct FakeClock {
    cur: Instant,
    auto_advance: Duration,
}
impl Default for FakeClock {
    fn default() -> Self {
        FakeClock {
            cur: Instant::now(),
            auto_advance: Duration::from_millis(1),
        }
    }
}
impl Clock for FakeClock {
    fn now(&mut self) -> Instant {
        self.cur += self.auto_advance;
        self.cur
    }
}
impl FakeClock {
    pub fn advance(&mut self, duration: Duration) {
        self.cur += duration;
    }
}
