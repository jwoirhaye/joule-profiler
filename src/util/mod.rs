use std::time::{Duration, SystemTime, UNIX_EPOCH};

use log::warn;

pub mod file;

pub fn get_timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|e| {
            warn!("System time is before UNIX_EPOCH: {}, using 0", e);
            Duration::from_secs(0)
        })
        .as_micros()
}
