use std::collections::HashMap;
use std::time::{Duration, Instant};

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use tracing::debug;

static LAST_KEY_UP: Lazy<Mutex<HashMap<u16, Instant>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static KEY_PRESSED: Lazy<Mutex<HashMap<u16, bool>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub struct ChatteringFilter {
    threshold: Duration,
}

impl ChatteringFilter {
    pub fn new(threshold_ms: u32) -> Self {
        Self {
            threshold: Duration::from_millis(threshold_ms as u64),
        }
    }

    pub fn should_forward(&self, code: u16, value: i32, timestamp: &Instant) -> bool {
        match value {
            2 => true,
            0 => {
                let mut last_up = LAST_KEY_UP.lock();
                last_up.insert(code, *timestamp);

                let mut pressed = KEY_PRESSED.lock();
                pressed.insert(code, false);
                true
            }
            1 => {
                let last_up = LAST_KEY_UP.lock();
                if let Some(last_up_instant) = last_up.get(&code) {
                    let elapsed = timestamp.duration_since(*last_up_instant);
                    if elapsed < self.threshold {
                        debug!("filtered chattering: code={}, elapsed={:?}", code, elapsed);
                        return false;
                    }
                }

                let mut pressed = KEY_PRESSED.lock();
                pressed.insert(code, true);
                true
            }
            _ => true,
        }
    }

    pub fn reset_state(&self) {
        LAST_KEY_UP.lock().clear();
        KEY_PRESSED.lock().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hold_events_always_forwarded() {
        let filter = ChatteringFilter::new(30);
        filter.reset_state();
        let timestamp = Instant::now();

        assert!(filter.should_forward(30, 2, &timestamp));
    }

    #[test]
    fn test_key_up_always_forwarded() {
        let filter = ChatteringFilter::new(30);
        filter.reset_state();
        let timestamp = Instant::now();

        assert!(filter.should_forward(30, 0, &timestamp));
    }

    #[test]
    fn test_first_key_down_forwarded() {
        let filter = ChatteringFilter::new(30);
        filter.reset_state();
        let timestamp = Instant::now();

        assert!(filter.should_forward(30, 1, &timestamp));
    }

    #[test]
    fn test_rapid_key_down_filtered() {
        let filter = ChatteringFilter::new(30);
        filter.reset_state();

        let base = Instant::now();
        let key_down1 = base;
        let key_up = base + Duration::from_millis(5);
        let key_down2 = base + Duration::from_millis(10);

        filter.should_forward(30, 1, &key_down1);
        filter.should_forward(30, 0, &key_up);
        assert!(!filter.should_forward(30, 1, &key_down2));
    }

    #[test]
    fn test_normal_key_press_forwarded() {
        let filter = ChatteringFilter::new(30);
        filter.reset_state();

        let base = Instant::now();
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

        let base = Instant::now();
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

        let base = Instant::now();
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

        let base = Instant::now();
        filter.should_forward(30, 1, &base);
        filter.should_forward(30, 0, &(base + Duration::from_millis(5)));
        assert!(filter.should_forward(31, 1, &(base + Duration::from_millis(5))));
    }
}
