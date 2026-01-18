use std::collections::HashSet;

use crate::{
    cli::{Cli, ProfilerCommand},
    output::{OutputFormat, output_format},
};

#[derive(Debug, Clone)]
pub enum Command {
    Profile(ProfileConfig),
    ListSensors(ListSensorsConfig),
}

#[derive(Debug, Clone)]
pub struct Config {
    pub mode: Command,
    pub rapl_path: Option<String>,
    pub output_format: OutputFormat,
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
                Command::Profile(ProfileConfig {
                    iterations: common.iterations.unwrap_or(1),
                    jouleit_file: common.jouleit_file,
                    output_file: common.output_file,
                    cmd: common.cmd,
                    rapl_polling: common.rapl_polling,
                    mode: Mode::SimpleMode,
                    sockets,
                })
            }
            ProfilerCommand::Phases(phases) => {
                let common = phases.common;
                Command::Profile(ProfileConfig {
                    iterations: common.iterations.unwrap_or(1),
                    jouleit_file: common.jouleit_file,
                    output_file: common.output_file,
                    cmd: common.cmd,
                    rapl_polling: common.rapl_polling,
                    mode: Mode::PhaseMode(PhasesConfig {
                        token_pattern: phases.token_pattern,
                    }),
                    sockets,
                })
            }

            ProfilerCommand::ListSensors(list) => Command::ListSensors(ListSensorsConfig {
                output_format: output_format(list.json, list.csv),
            }),
        };

        Config {
            mode,
            rapl_path: cli.rapl_path,
            output_format: output_format(cli.json, cli.csv),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProfileConfig {
    pub iterations: usize,
    pub jouleit_file: Option<String>,
    pub output_file: Option<String>,
    pub cmd: Vec<String>,
    pub sockets: Option<HashSet<u32>>,
    pub rapl_polling: Option<f64>,
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
}
