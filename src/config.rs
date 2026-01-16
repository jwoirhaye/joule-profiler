use crate::cli::{Cli, ProfilerCommand};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct ProfileConfig {
    pub iterations: usize,
    pub output_format: OutputFormat,
    pub jouleit_file: Option<String>,
    pub output_file: Option<String>,
    pub cmd: Vec<String>,
    pub sockets: Option<HashSet<u32>>,
    pub rapl_polling: Option<f64>,
    pub rapl_path: Option<String>,
    pub mode: Mode,
}

#[derive(Debug, Clone)]
pub enum Mode {
    SimpleMode,
    PhaseMode(PhasesConfig),
}

#[derive(Debug, Clone)]
pub struct PhasesConfig {
    pub token_pattern: String,
}

#[derive(Debug, Clone)]
pub struct ListSensorsConfig {
    pub output_format: OutputFormat,
    pub rapl_path: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Command {
    Profile(ProfileConfig),
    ListSensors(ListSensorsConfig),
}

#[derive(Debug, Clone)]
pub struct Config {
    pub mode: Command,
}

impl From<Cli> for Config {
    fn from(cli: Cli) -> Self {
        let sockets = cli.sockets.map(|s| {
            s.split(',')
                .filter_map(|x| x.trim().parse::<u32>().ok())
                .collect()
        });

        let mode = match cli.command {
            ProfilerCommand::Simple(simple) => {
                let common = simple.common;
                let output_format = output_format(common.json, common.csv);

                Command::Profile(ProfileConfig {
                    iterations: common.iterations.unwrap_or(1),
                    output_format,
                    jouleit_file: common.jouleit_file,
                    output_file: common.output_file,
                    cmd: common.cmd,
                    rapl_polling: common.rapl_polling,
                    rapl_path: cli.rapl_path,
                    mode: Mode::SimpleMode,
                    sockets,
                })
            }
            ProfilerCommand::Phases(phases) => {
                let common = phases.common;
                let output_format = output_format(common.json, common.csv);
                Command::Profile(ProfileConfig {
                    iterations: common.iterations.unwrap_or(1),
                    output_format,
                    jouleit_file: common.jouleit_file,
                    output_file: common.output_file,
                    cmd: common.cmd,
                    rapl_polling: common.rapl_polling,
                    rapl_path: cli.rapl_path,
                    mode: Mode::PhaseMode(PhasesConfig {
                        token_pattern: phases.token_pattern,
                    }),
                    sockets,
                })
            }

            ProfilerCommand::ListSensors(list) => Command::ListSensors(ListSensorsConfig {
                output_format: output_format(list.json, list.csv),
                rapl_path: cli.rapl_path,
            }),
        };

        Config { mode }
    }
}

#[derive(Debug, Clone)]
pub enum OutputFormat {
    Terminal,
    Json,
    Csv,
}

fn output_format(json: bool, csv: bool) -> OutputFormat {
    if json {
        OutputFormat::Json
    } else if csv {
        OutputFormat::Csv
    } else {
        OutputFormat::Terminal
    }
}
