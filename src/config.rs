use anyhow::Result;
use log::{debug, info, trace, warn};

use crate::cli::{PhasesArgs, SimpleArgs};
use crate::rapl::{RaplDomain, discover_sockets};

#[derive(Debug, Clone)]
pub struct Config {
    pub sockets: Vec<u32>,
    pub json: bool,
    pub csv: bool,
    pub iterations: Option<usize>,
    pub jouleit_file: Option<String>,
    pub output_file: Option<String>,
    pub token_start: Option<String>,
    pub token_end: Option<String>,
    pub cmd: Vec<String>,
}

impl Config {
    pub fn from_simple(args: SimpleArgs, domains: &[RaplDomain]) -> Result<Self> {
        info!("Building configuration from simple mode arguments");

        let sockets = parse_or_all_sockets(args.sockets.as_deref(), domains)?;

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
            "Simple mode config: sockets={:?}, json={}, csv={}, iterations={:?}, cmd={:?}",
            sockets, args.json, args.csv, args.iterations, args.cmd
        );

        if let Some(n) = args.iterations {
            info!("Configured for {} iteration(s)", n);
        }

        if let Some(ref file) = args.jouleit_file {
            debug!("Output file specified: {}", file);
        }

        Ok(Self {
            sockets,
            json: args.json,
            csv: args.csv,
            iterations: args.iterations,
            jouleit_file: args.jouleit_file,
            output_file: args.output_file,
            token_start: None,
            token_end: None,
            cmd: args.cmd,
        })
    }

    pub fn from_phases(args: PhasesArgs, domains: &[RaplDomain]) -> Result<Self> {
        info!("Building configuration from phases mode arguments");

        let sockets = parse_or_all_sockets(args.sockets.as_deref(), domains)?;

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

        debug!(
            "Phases mode config: sockets={:?}, json={}, csv={}, iterations={:?}, tokens=[{}, {}], cmd={:?}",
            sockets,
            args.json,
            args.csv,
            args.iterations,
            args.token_start,
            args.token_end,
            args.cmd
        );

        info!(
            "Phase tokens: start='{}', end='{}'",
            args.token_start, args.token_end
        );

        if let Some(n) = args.iterations {
            info!("Configured for {} iteration(s)", n);
        }

        if let Some(ref file) = args.jouleit_file {
            debug!("Output file specified: {}", file);
        }

        Ok(Self {
            sockets,
            json: args.json,
            csv: args.csv,
            iterations: args.iterations,
            jouleit_file: args.jouleit_file,
            output_file: args.output_file,
            token_start: Some(args.token_start),
            token_end: Some(args.token_end),
            cmd: args.cmd,
        })
    }

    pub fn output_format(&self) -> OutputFormat {
        if self.json {
            trace!("Output format determined: JSON");
            OutputFormat::Json
        } else if self.csv {
            trace!("Output format determined: CSV");
            OutputFormat::Csv
        } else {
            trace!("Output format determined: Terminal (default)");
            OutputFormat::Terminal
        }
    }

    pub fn is_multi_iteration(&self) -> bool {
        self.iterations.map_or(false, |n| n > 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Csv,
    Terminal,
}

fn parse_or_all_sockets(spec: Option<&str>, domains: &[RaplDomain]) -> Result<Vec<u32>> {
    if let Some(spec) = spec {
        debug!("Parsing socket specification: {}", spec);
        let sockets = crate::rapl::parse_sockets(spec, domains)?;
        info!("Using specified sockets: {:?}", sockets);
        Ok(sockets)
    } else {
        let sockets = discover_sockets(domains);
        debug!(
            "No socket specification, using all discovered sockets: {:?}",
            sockets
        );
        Ok(sockets)
    }
}
