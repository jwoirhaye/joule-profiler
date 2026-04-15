use std::collections::HashMap;

use anyhow::Result;

use joule_profiler_core::{JouleProfiler, source::MetricReader};
use source_nvml::config::NvmlConfig;
use source_perf_event::config::PerfConfig;
use source_rapl::{perf::config::RaplPerfConfig, powercap::config::RaplPowercapConfig};

use crate::{CliArgs, RaplBackend};

#[macro_export]
macro_rules! register_sources {
    ($profiler:expr, $configs:expr, [$($source:ty),* $(,)?]) => {
        $(register_source::<$source>($profiler, $configs)?;)*
    };
}

pub struct ConfigTable<'a> {
    cli: &'a CliArgs,
    inner: HashMap<String, toml::Value>,
}

impl<'a> ConfigTable<'a> {
    pub fn new(table: toml::Table, cli: &'a CliArgs) -> Self {
        Self {
            inner: table.into_iter().collect(),
            cli,
        }
    }

    pub fn get_config<R>(&mut self) -> Result<Option<R::Config>>
    where
        R: MetricReader,
        R::Config: CliOverride + Default,
    {
        let (base_config, from_toml) = match self.inner.remove(R::get_id()) {
            Some(value) => (value.try_into()?, true),
            None => (R::Config::default(), false),
        };

        Ok(base_config.override_config(self.cli, from_toml))
    }
}

pub fn register_source<R>(profiler: &mut JouleProfiler, configs: &mut ConfigTable) -> Result<()>
where
    R: MetricReader,
    R::Config: CliOverride,
{
    if let Some(config) = configs.get_config::<R>()? {
        let reader = R::from_config(config)?;
        profiler.add_source(reader);
    }

    Ok(())
}

pub trait CliOverride: Sized {
    #[allow(unused_variables)]
    fn override_config(self, cli: &CliArgs, from_toml: bool) -> Option<Self> {
        if from_toml { Some(self) } else { None }
    }
}

impl CliOverride for () {
    fn override_config(self, _cli: &CliArgs, _from_toml: bool) -> Option<Self> {
        None
    }
}

impl CliOverride for NvmlConfig {
    fn override_config(self, cli: &CliArgs, from_toml: bool) -> Option<Self> {
        if cli.gpu || from_toml {
            Some(self)
        } else {
            None
        }
    }
}

impl CliOverride for PerfConfig {
    fn override_config(self, cli: &CliArgs, from_toml: bool) -> Option<Self> {
        if cli.perf || from_toml {
            Some(self)
        } else {
            None
        }
    }
}

impl CliOverride for RaplPowercapConfig {
    fn override_config(self, cli: &CliArgs, _from_toml: bool) -> Option<Self> {
        match cli.rapl_backend {
            RaplBackend::Powercap => Some(self),
            RaplBackend::Perf => None,
        }
    }
}

impl CliOverride for RaplPerfConfig {
    fn override_config(self, cli: &CliArgs, _from_toml: bool) -> Option<Self> {
        match cli.rapl_backend {
            RaplBackend::Powercap => Some(self),
            RaplBackend::Perf => None,
        }
    }
}
