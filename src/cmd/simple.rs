use anyhow::Result;
use log::{debug, info};

use crate::cli::SimpleArgs;
use crate::config::{Config, OutputFormat};
use crate::errors::JouleProfilerError;
use crate::measure::MeasurementResult;
use crate::measure::measure_once;
use crate::output::csv::CsvOutput;
use crate::output::{JsonOutput, OutputFormat as OutputFormatTrait, TerminalOutput};
use crate::rapl::RaplDomain;

/// Runs the profiler in simple mode.
pub fn run_simple(args: SimpleArgs, domains: &[RaplDomain]) -> Result<()> {
    info!("Running simple mode");
    let config = Config::from_simple(args, domains)?;

    if let Some(n) = config.iterations {
        debug!("Simple mode with {} iteration(s)", n);
        run_simple_iterations(&config, domains, n)
    } else {
        debug!("Simple mode with single measurement");
        run_simple_single(&config, domains)
    }
}

/// Executes a single measurement and outputs the result.
fn run_simple_single(config: &Config, domains: &[RaplDomain]) -> Result<()> {
    info!("Measuring single execution");
    let res: MeasurementResult = measure_once(config, domains)?;

    debug!("Measurement complete, formatting output");

    match config.output_format() {
        OutputFormat::Json => {
            debug!("Using JSON output format (file)");
            let mut out = JsonOutput::new(config)?;
            out.simple_single(&config, &res)?;
        }
        OutputFormat::Csv => {
            debug!("Using CSV output format (file)");
            let mut out = CsvOutput::new(config)?;
            out.simple_single(&config, &res)?;
        }
        OutputFormat::Terminal => {
            debug!("Using terminal output format");
            let mut out = TerminalOutput::new();
            out.simple_single(&config, &res)?;
        }
    }

    info!("Simple single measurement completed successfully");

    std::process::exit(res.exit_code);
}

/// Executes multiple measurements (iterations) and outputs aggregated results.
fn run_simple_iterations(config: &Config, domains: &[RaplDomain], iterations: usize) -> Result<()> {
    if iterations == 0 {
        return Err(JouleProfilerError::InvalidIterations(0).into());
    }

    info!("Running {} iteration(s) in simple mode", iterations);
    let mut results = Vec::with_capacity(iterations);

    for i in 0..iterations {
        info!("═══ Iteration {}/{} ═══", i + 1, iterations);
        let res = measure_once(config, domains)?;
        debug!(
            "Iteration {} completed: {} µJ total, duration {} ms, exit code {}",
            i + 1,
            res.energy_uj.values().sum::<u64>(),
            res.duration_ms,
            res.exit_code
        );
        results.push((i, res));
    }

    info!("All {} iteration(s) completed successfully", iterations);
    debug!("Formatting output");

    match config.output_format() {
        OutputFormat::Json => {
            debug!("Using JSON output format (file)");
            let mut out = JsonOutput::new(config)?;
            out.simple_iterations(config, &results)?;
        }
        OutputFormat::Csv => {
            debug!("Using CSV output format (file)");
            let mut out = CsvOutput::new(config)?;
            out.simple_iterations(config, &results)?;
        }
        OutputFormat::Terminal => {
            debug!("Using terminal output format");
            let mut out = TerminalOutput::new();
            out.simple_iterations(config, &results)?;
        }
    }

    info!("Simple iterations completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_mock_domain(name: &str, socket: u32) -> RaplDomain {
        RaplDomain {
            path: PathBuf::from(format!(
                "/sys/class/powercap/intel-rapl:0/{}/energy_uj",
                name
            )),
            name: name.to_string(),
            socket,
            max_energy_uj: Some(10_000_000),
        }
    }

    fn create_test_config(
        cmd: Vec<String>,
        sockets: Vec<u32>,
        iterations: Option<usize>,
    ) -> Config {
        Config {
            sockets,
            json: false,
            csv: false,
            iterations,
            jouleit_file: None,
            output_file: None,
            token_start: None,
            token_end: None,
            cmd,
        }
    }

    #[test]
    fn test_run_simple_iterations_zero() {
        let config = create_test_config(
            vec!["echo".to_string(), "test".to_string()],
            vec![0],
            Some(0),
        );
        let domains = vec![create_mock_domain("package-0", 0)];

        let result = run_simple_iterations(&config, &domains, 0);
        assert!(result.is_err());

        if let Err(e) = result {
            let err = e.downcast::<JouleProfilerError>().unwrap();
            assert!(matches!(err, JouleProfilerError::InvalidIterations(0)));
        }
    }

    #[test]
    fn test_run_simple_iterations_validates_count() {
        let config = create_test_config(vec!["true".to_string()], vec![0], Some(1));
        let domains = vec![create_mock_domain("package-0", 0)];

        let result = run_simple_iterations(&config, &domains, 1);

        if let Err(e) = result {
            let err_msg = format!("{}", e);
            assert!(!err_msg.contains("InvalidIterations"));
        }
    }

    #[test]
    fn test_config_from_simple_validates_iterations() {
        use crate::cli::SimpleArgs;

        let args = SimpleArgs {
            json: false,
            csv: false,
            iterations: Some(0),
            jouleit_file: None,
            output_file: None,
            sockets: None,
            cmd: vec!["echo".to_string()],
        };

        let domains = vec![create_mock_domain("package-0", 0)];
        let result = Config::from_simple(args, &domains);

        assert!(result.is_err());
    }

    #[test]
    fn test_iterations_capacity_allocation() {
        let iterations = 100;
        let mut results = Vec::with_capacity(iterations);

        assert_eq!(results.capacity(), 100);
        assert_eq!(results.len(), 0);

        for i in 0..iterations {
            results.push((i, "test"));
        }

        assert_eq!(results.len(), 100);
        assert!(results.capacity() >= 100);
    }

    #[test]
    fn test_output_format_detection() {
        let test_cases = vec![
            (true, false, OutputFormat::Json),
            (false, true, OutputFormat::Csv),
            (false, false, OutputFormat::Terminal),
        ];

        for (json, csv, expected) in test_cases {
            let config = Config {
                sockets: vec![0],
                json,
                csv,
                iterations: None,
                jouleit_file: None,
                output_file: None,
                token_start: None,
                token_end: None,
                cmd: vec!["echo".to_string()],
            };

            assert_eq!(
                config.output_format(),
                expected,
                "json={}, csv={} should result in {:?}",
                json,
                csv,
                expected
            );
        }
    }
}
