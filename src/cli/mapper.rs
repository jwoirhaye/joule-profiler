use crate::cli::{CliArgs, ProfilerCommand};
use crate::config::{Command, Config, ListSensorsConfig, ProfileConfig};
use crate::output::output_format;
use std::collections::HashSet;

pub fn cli_to_config(cli_args: CliArgs) -> Config {
    let sockets: Option<HashSet<u32>> = cli_args.sockets.map(|s| {
        s.split(',')
            .filter_map(|x| x.trim().parse::<u32>().ok())
            .collect()
    });

    let command = match cli_args.command {
        ProfilerCommand::Phases(phases) => Command::Profile(ProfileConfig {
            iterations: phases.iterations.unwrap_or(1),
            stdout_file: phases.stdout_file,
            cmd: phases.cmd,
            rapl_polling: phases.rapl_polling,
            token_pattern: phases.token_pattern,
            sockets,
        }),

        ProfilerCommand::ListSensors(list) => Command::ListSensors(ListSensorsConfig {
            output_format: output_format(list.json, list.csv),
        }),
    };

    Config {
        command,
        rapl_path: cli_args.rapl_path,
        output_format: output_format(cli_args.json, cli_args.csv),
        output_file: cli_args.output_file,
    }
}
