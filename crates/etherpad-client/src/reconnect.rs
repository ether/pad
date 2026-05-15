use std::time::Duration;

/// Exponential backoff: 1s, 2s, 4s, 8s, 16s, then capped at 30s.
#[derive(Debug, Clone)]
pub struct Reconnect {
    attempt: u32,
    cap: Duration,
}

impl Reconnect {
    pub fn new() -> Self {
        Self {
            attempt: 0,
            cap: Duration::from_secs(30),
        }
    }

    /// Delay to wait *before* the next attempt. attempt 0 returns 1s.
    pub fn next_delay(&mut self) -> Duration {
        let secs = 1u64 << self.attempt.min(5);
        let d = Duration::from_secs(secs);
        self.attempt += 1;
        d.min(self.cap)
    }

    /// Call after a successful connection — resets the backoff.
    pub fn reset(&mut self) {
        self.attempt = 0;
    }

    pub fn attempt(&self) -> u32 {
        self.attempt
    }
}

impl Default for Reconnect {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_sequence() {
        let mut r = Reconnect::new();
        assert_eq!(r.next_delay(), Duration::from_secs(1));
        assert_eq!(r.next_delay(), Duration::from_secs(2));
        assert_eq!(r.next_delay(), Duration::from_secs(4));
        assert_eq!(r.next_delay(), Duration::from_secs(8));
        assert_eq!(r.next_delay(), Duration::from_secs(16));
        assert_eq!(r.next_delay(), Duration::from_secs(30));
        assert_eq!(r.next_delay(), Duration::from_secs(30));
    }

    #[test]
    fn reset_clears_backoff() {
        let mut r = Reconnect::new();
        for _ in 0..4 {
            r.next_delay();
        }
        r.reset();
        assert_eq!(r.next_delay(), Duration::from_secs(1));
    }
}
