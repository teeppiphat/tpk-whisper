use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Sliding-window limiter. typhoon-asr-realtime allows 100 reqs/minute.
pub struct RateLimiter {
    window: Duration,
    max: usize,
    hits: VecDeque<Instant>,
}

impl RateLimiter {
    pub fn new(max: usize, window: Duration) -> Self {
        Self {
            window,
            max,
            hits: VecDeque::new(),
        }
    }

    /// Records and allows a request if under the limit; returns false otherwise.
    pub fn try_acquire(&mut self) -> bool {
        let now = Instant::now();
        while let Some(&front) = self.hits.front() {
            if now.duration_since(front) > self.window {
                self.hits.pop_front();
            } else {
                break;
            }
        }
        if self.hits.len() < self.max {
            self.hits.push_back(now);
            true
        } else {
            false
        }
    }
}
