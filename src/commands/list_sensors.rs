use anyhow::Result;

use crate::{
    config::list_sensors::ListSensorsConfig,
    core::{displayer::ListSensorsDisplayer, sensor::Sensor, source::MetricReader},
    sources::rapl::init_rapl,
};

pub fn run_list_sensors(config: &ListSensorsConfig) -> Result<()> {
    let mut displayer: Box<dyn ListSensorsDisplayer> = config.try_into()?;

    let sources = [init_rapl(config.rapl_path.as_deref(), None, None)?];

    let sensors: Vec<Sensor> = sources
        .iter()
        .flat_map(|source| source.get_sensors())
        .flatten()
        .collect();

    displayer.list_sensors(&sensors)?;

    Ok(())
}
