use anyhow::Result;

use crate::{
    config::ListSensorsConfig,
    output::{Displayer, OutputFormatTrait},
    source::{MetricReader, rapl::init_rapl},
};

pub fn run_list_sensors(config: &ListSensorsConfig) -> Result<()> {
    let sources = vec![init_rapl(config.rapl_path.as_deref(), None, None)?];

    let sensors: Vec<_> = sources
        .iter()
        .flat_map(|source| source.get_sensors())
        .flatten()
        .collect();

    let mut displayer = Displayer::try_from(config)?;
    displayer.list_sensors(config, &sensors)?;
    Ok(())
}
