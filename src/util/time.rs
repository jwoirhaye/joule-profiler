use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Get the current system timestamp.
pub fn get_timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_micros()
}
