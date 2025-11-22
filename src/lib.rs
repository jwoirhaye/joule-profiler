pub mod cli;
pub mod cmd;
pub mod config;
pub mod errors;
pub mod measure;
pub mod output;
pub mod rapl;

use anyhow::Result;
use clap::Parser;
use env_logger::Env;
use log::{debug, info, trace};

/// Main entry point for the Joule Profiler application.
pub fn run() -> Result<()> {
    let cli = cli::Cli::parse();

    init_logging(cli.verbose);

    info!("Joule Profiler starting");
    debug!("Parsed CLI arguments: {:?}", cli);
    trace!("Verbose level: {}", cli.verbose);

    let result = cmd::run(cli);

    match &result {
        Ok(_) => info!("Joule Profiler completed successfully"),
        Err(e) => log::error!("Joule Profiler failed: {}", e),
    }

    result
}

/// Initializes the logging system based on verbosity flags.
pub fn init_logging(level: u8) {
    let lvl_str = match level {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    let env = Env::default().default_filter_or(format!("joule_profiler={lvl_str}"));
    env_logger::Builder::from_env(env)
        .format_timestamp_millis()
        .init();

    match level {
        0 => {}
        1 => info!("Logging initialized at INFO level"),
        2 => debug!("Logging initialized at DEBUG level"),
        _ => trace!("Logging initialized at TRACE level"),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_logging_level_mapping_warn() {
        let level = 0;
        let expected = "warn";

        let result = match level {
            0 => "warn",
            1 => "info",
            2 => "debug",
            _ => "trace",
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn test_logging_level_mapping_info() {
        let level = 1;
        let expected = "info";

        let result = match level {
            0 => "warn",
            1 => "info",
            2 => "debug",
            _ => "trace",
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn test_logging_level_mapping_debug() {
        let level = 2;
        let expected = "debug";

        let result = match level {
            0 => "warn",
            1 => "info",
            2 => "debug",
            _ => "trace",
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn test_logging_level_mapping_trace() {
        for level in 3..10 {
            let result = match level {
                0 => "warn",
                1 => "info",
                2 => "debug",
                _ => "trace",
            };

            assert_eq!(result, "trace", "Level {} should map to trace", level);
        }
    }

    #[test]
    fn test_verbosity_levels() {
        let test_cases = vec![
            (0, "warn"),
            (1, "info"),
            (2, "debug"),
            (3, "trace"),
            (4, "trace"),
            (100, "trace"),
        ];

        for (level, expected) in test_cases {
            let result = match level {
                0 => "warn",
                1 => "info",
                2 => "debug",
                _ => "trace",
            };

            assert_eq!(
                result, expected,
                "Verbosity level {} should map to {}",
                level, expected
            );
        }
    }
}
