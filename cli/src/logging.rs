use env_logger::Builder;
use log::{LevelFilter, debug, info, trace};

/// Initializes the logging system based on verbosity flags.
pub fn init_logging(level: u8) {
    let level_filter = match level {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    Builder::new().filter_level(level_filter).init();

    match level {
        0 => {}
        1 => info!("Logging initialized at INFO level"),
        2 => debug!("Logging initialized at DEBUG level"),
        _ => trace!("Logging initialized at TRACE level"),
    }
}
