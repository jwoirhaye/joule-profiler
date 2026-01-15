use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Get the current system timestamp.
pub fn get_timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_micros()
}

/// Convert a frenquency to a duration in nanos.
pub fn duration_from_hz(hz: u64) -> Duration {
    assert!(hz > 0);
    Duration::from_nanos(1_000_000_000 / hz)
}
