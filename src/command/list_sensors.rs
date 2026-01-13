use anyhow::Result;

use crate::{
    config::ListSensorsConfig,
    output::{Displayer, OutputFormatTrait},
    source::SourceManager,
};

pub fn run_list_sensors(manager: &SourceManager, config: &ListSensorsConfig) -> Result<()> {
    let sensors: Vec<_> = manager.list_sensors()?;
    let mut displayer = Displayer::try_from(config)?;
    displayer.list_sensors(config, &sensors)?;
    Ok(())
}
