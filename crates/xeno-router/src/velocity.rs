//! Token generation velocity calculator with Exponential Moving Average (EMA) and sliding window.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Real-time token generation velocity calculator.
#[derive(Debug, Clone)]
pub struct TokenVelocityCalculator {
    /// Rolling window sliding duration (default: 1.0s).
    window_duration: Duration,
    /// Ring buffer storing `(timestamp, token_count)` events.
    window_events: VecDeque<(Instant, usize)>,
    /// Exponential moving average smoothing factor alpha (0.0 to 1.0, default: 0.2).
    alpha: f64,
    /// Current smoothed EMA velocity in tokens/second.
    current_ema: f64,
    /// Instant when generation began.
    start_time: Instant,
    /// Timestamp of the last recorded token chunk.
    last_update: Option<Instant>,
    /// Total tokens accumulated.
    total_tokens: usize,
}

impl Default for TokenVelocityCalculator {
    fn default() -> Self {
        Self::new(Duration::from_millis(1000), 0.2)
    }
}

impl TokenVelocityCalculator {
    /// Constructs a new [`TokenVelocityCalculator`] with custom window duration and EMA alpha factor.
    pub fn new(window_duration: Duration, alpha: f64) -> Self {
        let now = Instant::now();
        Self {
            window_duration,
            window_events: VecDeque::new(),
            alpha: alpha.clamp(0.01, 1.0),
            current_ema: 0.0,
            start_time: now,
            last_update: None,
            total_tokens: 0,
        }
    }

    /// Records newly arrived tokens from a stream chunk.
    pub fn record_tokens(&mut self, count: usize) {
        if count == 0 {
            return;
        }

        let now = Instant::now();
        self.total_tokens = self.total_tokens.saturating_add(count);
        self.window_events.push_back((now, count));

        // Update EMA
        if let Some(prev_time) = self.last_update {
            let dt = (now - prev_time).as_secs_f64();
            if dt > 0.0001 {
                let instant_rate = (count as f64) / dt;
                if self.current_ema == 0.0 {
                    self.current_ema = instant_rate;
                } else {
                    self.current_ema = (self.alpha * instant_rate) + ((1.0 - self.alpha) * self.current_ema);
                }
            }
        } else {
            let dt = (now - self.start_time).as_secs_f64().max(0.001);
            self.current_ema = (count as f64) / dt;
        }

        self.last_update = Some(now);
        self.prune_window(now);
    }

    /// Prunes events older than the sliding window horizon.
    fn prune_window(&mut self, now: Instant) {
        let cutoff = now.checked_sub(self.window_duration).unwrap_or(self.start_time);
        while let Some(&(ts, _)) = self.window_events.front() {
            if ts < cutoff {
                self.window_events.pop_front();
            } else {
                break;
            }
        }
    }

    /// Calculates current sliding window token velocity in tokens/second.
    pub fn current_velocity(&mut self) -> f64 {
        let now = Instant::now();
        self.prune_window(now);

        if self.window_events.is_empty() {
            return 0.0;
        }

        let window_tokens: usize = self.window_events.iter().map(|(_, c)| *c).sum();
        let earliest = self.window_events.front().map(|(t, _)| *t).unwrap_or(self.start_time);
        let dt = (now - earliest).as_secs_f64();

        if dt < 0.001 {
            // Very short time window
            (window_tokens as f64) / 0.001
        } else {
            (window_tokens as f64) / dt
        }
    }

    /// Returns the smoothed Exponential Moving Average (EMA) velocity in tokens/second.
    pub fn ema_velocity(&self) -> f64 {
        self.current_ema
    }

    /// Returns the lifetime average generation velocity from the start in tokens/second.
    pub fn average_velocity(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed < 0.001 || self.total_tokens == 0 {
            0.0
        } else {
            (self.total_tokens as f64) / elapsed
        }
    }

    /// Returns the total tokens recorded.
    pub fn total_tokens(&self) -> usize {
        self.total_tokens
    }

    /// Returns the total elapsed duration since generation started.
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Resets all counters and timers to initial state.
    pub fn reset(&mut self) {
        let now = Instant::now();
        self.window_events.clear();
        self.current_ema = 0.0;
        self.start_time = now;
        self.last_update = None;
        self.total_tokens = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_velocity_zero_tokens() {
        let mut calc = TokenVelocityCalculator::default();
        assert_eq!(calc.current_velocity(), 0.0);
        assert_eq!(calc.average_velocity(), 0.0);
        assert_eq!(calc.total_tokens(), 0);
    }

    #[test]
    fn test_velocity_recording_and_reset() {
        let mut calc = TokenVelocityCalculator::new(Duration::from_millis(500), 0.5);
        calc.record_tokens(10);
        calc.record_tokens(20);

        assert_eq!(calc.total_tokens(), 30);
        assert!(calc.ema_velocity() > 0.0);

        calc.reset();
        assert_eq!(calc.total_tokens(), 0);
        assert_eq!(calc.ema_velocity(), 0.0);
    }
}
