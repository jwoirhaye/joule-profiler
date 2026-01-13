use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn get_timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_micros()
}

pub fn _duration_from_hz(hz: u64) -> Duration {
    assert!(hz > 0);

    let nanos = (1_000_000_000.0 / hz as f64).round() as u64;
    Duration::from_nanos(nanos)
}
