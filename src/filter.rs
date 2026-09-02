use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use parking_lot::Mutex;
use tracing::info;

pub struct ChatteringFilter {
    threshold: Duration,
    last_key_up: Mutex<HashMap<u16, SystemTime>>,
    key_pressed: Mutex<HashMap<u16, bool>>,
}

impl ChatteringFilter {
    pub fn new(threshold_ms: u32) -> Self {
        Self {
            threshold: Duration::from_millis(threshold_ms as u64),
            last_key_up: Mutex::new(HashMap::new()),
            key_pressed: Mutex::new(HashMap::new()),
        }
    }

    pub fn should_forward(&self, code: u16, value: i32, timestamp: &SystemTime) -> bool {
        match value {
            2 => true,
            0 => {
                let mut pressed = self.key_pressed.lock();
                if !pressed.get(&code).copied().unwrap_or(false) {
                    info!("filtered duplicate key-up: code={}", code);
                    return false;
                }

                let mut last_up = self.last_key_up.lock();
                last_up.insert(code, *timestamp);
                pressed.insert(code, false);
                true
            }
            1 => {
                let last_up = self.last_key_up.lock();
                let mut pressed = self.key_pressed.lock();

                if pressed.get(&code).copied().unwrap_or(false) {
                    info!("filtered duplicate key-down: code={}", code);
                    return false;
                }

                if let Some(last_up_instant) = last_up.get(&code) {
                    let elapsed = timestamp
                        .duration_since(*last_up_instant)
                        .unwrap_or_default();
                    if elapsed < self.threshold {
                        info!("filtered chattering: code={}, elapsed={:?}", code, elapsed);
                        return false;
                    }
                }

                pressed.insert(code, true);
                true
            }
            _ => true,
        }
    }

    pub fn reset_state(&self) {
        self.last_key_up.lock().clear();
        self.key_pressed.lock().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hold_events_always_forwarded() {
        let filter = ChatteringFilter::new(30);
        filter.reset_state();
        let timestamp = SystemTime::UNIX_EPOCH;

        assert!(filter.should_forward(30, 2, &timestamp));
    }

    #[test]
    fn test_key_up_after_key_down_forwarded() {
        let filter = ChatteringFilter::new(30);
        filter.reset_state();
        let timestamp = SystemTime::UNIX_EPOCH;

        filter.should_forward(30, 1, &timestamp);
        assert!(filter.should_forward(30, 0, &timestamp));
    }

    #[test]
    fn test_first_key_down_forwarded() {
        let filter = ChatteringFilter::new(30);
        filter.reset_state();
        let timestamp = SystemTime::UNIX_EPOCH;

        assert!(filter.should_forward(30, 1, &timestamp));
    }

    #[test]
    fn test_rapid_key_down_filtered() {
        let filter = ChatteringFilter::new(30);
        filter.reset_state();

        let base = SystemTime::UNIX_EPOCH;
        let key_down1 = base;
        let key_up = base + Duration::from_millis(5);
        let key_down2 = base + Duration::from_millis(10);

        filter.should_forward(30, 1, &key_down1);
        filter.should_forward(30, 0, &key_up);
        assert!(!filter.should_forward(30, 1, &key_down2));
    }

    #[test]
    fn test_repeated_key_down_filtered() {
        let filter = ChatteringFilter::new(30);
        filter.reset_state();

        let base = SystemTime::UNIX_EPOCH;
        assert!(filter.should_forward(30, 1, &base));
        assert!(!filter.should_forward(30, 1, &(base + Duration::from_millis(1))));
    }

    #[test]
    fn test_normal_key_press_forwarded() {
        let filter = ChatteringFilter::new(30);
        filter.reset_state();

        let base = SystemTime::UNIX_EPOCH;
        let key_down1 = base;
        let key_up = base + Duration::from_millis(5);
        let key_down2 = base + Duration::from_millis(50);

        filter.should_forward(30, 1, &key_down1);
        filter.should_forward(30, 0, &key_up);
        assert!(filter.should_forward(30, 1, &key_down2));
    }

    #[test]
    fn test_threshold_boundary_above() {
        let filter = ChatteringFilter::new(30);
        filter.reset_state();

        let base = SystemTime::UNIX_EPOCH;
        let key_down1 = base;
        let key_up = base + Duration::from_millis(35);
        let key_down2 = base + Duration::from_millis(66);

        filter.should_forward(30, 1, &key_down1);
        filter.should_forward(30, 0, &key_up);
        assert!(filter.should_forward(30, 1, &key_down2));
    }

    #[test]
    fn test_threshold_boundary_below() {
        let filter = ChatteringFilter::new(30);
        filter.reset_state();

        let base = SystemTime::UNIX_EPOCH;
        let key_down1 = base;
        let key_up = base + Duration::from_millis(35);
        let key_down2 = base + Duration::from_millis(64);

        filter.should_forward(30, 1, &key_down1);
        filter.should_forward(30, 0, &key_up);
        assert!(!filter.should_forward(30, 1, &key_down2));
    }

    #[test]
    fn test_different_keys_independent() {
        let filter = ChatteringFilter::new(30);
        filter.reset_state();

        let base = SystemTime::UNIX_EPOCH;
        filter.should_forward(30, 1, &base);
        filter.should_forward(30, 0, &(base + Duration::from_millis(5)));
        assert!(filter.should_forward(31, 1, &(base + Duration::from_millis(5))));
    }
}
