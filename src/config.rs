use std::fmt::Display;

use anyhow::Result;
use log::{debug, info, trace, warn};

use crate::cli::{PhasesArgs, SimpleArgs};

#[derive(Debug, Clone)]
pub struct Config {
    pub output_format: OutputFormat,
    pub iterations: Option<usize>,
    pub jouleit_file: Option<String>,
    pub output_file: Option<String>,
    pub token_pattern: Option<String>, // Remplace token_start et token_end
    pub cmd: Vec<String>,
}

impl Config {
    pub fn from_simple(args: SimpleArgs) -> Result<Self> {
        info!("Building configuration from simple mode arguments");

        if args.iterations == Some(0) {
            warn!("Invalid iterations value: 0");
            anyhow::bail!("Iterations must be >= 1");
        }

        if args.cmd.is_empty() {
            warn!("No command specified");
            anyhow::bail!("No command specified for simple mode");
        }

        if args.json && args.csv {
            warn!("Both --json and --csv flags provided");
            anyhow::bail!("Cannot use both --json and --csv flags simultaneously");
        }

        debug!(
            "Simple mode config: json={}, csv={}, iterations={:?}, cmd={:?}",
            args.json, args.csv, args.iterations, args.cmd
        );

        if let Some(n) = args.iterations {
            info!("Configured for {} iteration(s)", n);
        }

        if let Some(ref file) = args.jouleit_file {
            debug!("Output file specified: {}", file);
        }

        let output_format = OutputFormat::new(args.json, args.csv);
        trace!("Output format determined: {}", output_format);

        Ok(Self {
            output_format,
            iterations: args.iterations,
            jouleit_file: args.jouleit_file,
            output_file: args.output_file,
            token_pattern: None, // Pas de pattern en mode simple
            cmd: args.cmd,
        })
    }

    pub fn from_phases(args: PhasesArgs) -> Result<Self> {
        info!("Building configuration from phases mode arguments");

        if args.iterations == Some(0) {
            warn!("Invalid iterations value: 0");
            anyhow::bail!("Iterations must be >= 1");
        }

        if args.cmd.is_empty() {
            warn!("No command specified");
            anyhow::bail!("No command specified for phases mode");
        }

        if args.json && args.csv {
            warn!("Both --json and --csv flags provided");
            anyhow::bail!("Cannot use both --json and --csv flags simultaneously");
        }

        // Validation du pattern regex
        if let Err(e) = regex::Regex::new(&args.token_pattern) {
            warn!("Invalid regex pattern '{}': {}", args.token_pattern, e);
            anyhow::bail!("Invalid regex pattern '{}': {}", args.token_pattern, e);
        }

        debug!(
            "Phases mode config: json={}, csv={}, iterations={:?}, pattern='{}', cmd={:?}",
            args.json, args.csv, args.iterations, args.token_pattern, args.cmd
        );

        info!("Phase token pattern: '{}'", args.token_pattern);

        if let Some(n) = args.iterations {
            info!("Configured for {} iteration(s)", n);
        }

        if let Some(ref file) = args.jouleit_file {
            debug!("Output file specified: {}", file);
        }

        let output_format = OutputFormat::new(args.json, args.csv);
        trace!("Output format determined: {}", output_format);

        Ok(Self {
            output_format,
            iterations: args.iterations,
            jouleit_file: args.jouleit_file,
            output_file: args.output_file,
            token_pattern: Some(args.token_pattern), // Pattern regex
            cmd: args.cmd,
        })
    }

    pub fn is_multi_iteration(&self) -> bool {
        self.iterations.is_some_and(|n| n > 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Csv,
    Terminal,
}

impl Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            OutputFormat::Json => "JSON",
            OutputFormat::Csv => "CSV",
            OutputFormat::Terminal => "Terminal (default)",
        })
    }
}

impl OutputFormat {
    pub fn new(json: bool, csv: bool) -> OutputFormat {
        if json {
            OutputFormat::Json
        } else if csv {
            OutputFormat::Csv
        } else {
            OutputFormat::Terminal
        }
    }
}
