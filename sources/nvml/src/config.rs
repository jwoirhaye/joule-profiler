use std::collections::HashSet;

use serde::{Deserialize, Deserializer};

#[derive(Default, Deserialize, Debug)]
pub struct NvmlConfig {
    #[serde(default)]
    pub target_gpus: GpuSelector,

    #[serde(default)]
    pub exit_on_device_failure: bool,
}

#[derive(Debug, Clone, Default)]
pub enum GpuSelector {
    #[default]
    All,
    List(HashSet<u32>),
}

impl<'de> Deserialize<'de> for GpuSelector {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        if s.trim() == "*" {
            return Ok(GpuSelector::All);
        }
        let ids = s
            .split(',')
            .map(|part| part.trim().parse::<u32>().map_err(serde::de::Error::custom))
            .collect::<std::result::Result<HashSet<_>, _>>()?;
        Ok(GpuSelector::List(ids))
    }
}
