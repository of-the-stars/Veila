use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub(crate) struct KeyRepeatState {
    next_at: Instant,
    interval: Duration,
}

impl KeyRepeatState {
    pub(crate) fn new(now: Instant, initial_delay: Duration, interval: Duration) -> Self {
        Self {
            next_at: now + initial_delay,
            interval,
        }
    }

    pub(crate) fn due_in(&self, now: Instant) -> Duration {
        self.next_at.saturating_duration_since(now)
    }

    pub(crate) fn consume_if_due(&mut self, now: Instant) -> bool {
        if now < self.next_at {
            return false;
        }

        self.next_at = now + self.interval;
        true
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::KeyRepeatState;

    #[test]
    fn due_in_counts_down_to_zero() {
        let now = Instant::now();
        let state = KeyRepeatState::new(now, Duration::from_millis(300), Duration::from_millis(32));

        assert!(state.due_in(now) >= Duration::from_millis(299));
        assert_eq!(
            state.due_in(now + Duration::from_millis(300)),
            Duration::ZERO
        );
    }

    #[test]
    fn consume_if_due_waits_until_deadline() {
        let now = Instant::now();
        let mut state =
            KeyRepeatState::new(now, Duration::from_millis(300), Duration::from_millis(32));

        assert!(!state.consume_if_due(now + Duration::from_millis(299)));
        assert!(state.consume_if_due(now + Duration::from_millis(300)));
    }

    #[test]
    fn consume_if_due_rearms_to_interval() {
        let now = Instant::now();
        let mut state =
            KeyRepeatState::new(now, Duration::from_millis(300), Duration::from_millis(32));

        assert!(state.consume_if_due(now + Duration::from_millis(300)));
        assert!(!state.consume_if_due(now + Duration::from_millis(331)));
        assert!(state.consume_if_due(now + Duration::from_millis(332)));
    }
}
