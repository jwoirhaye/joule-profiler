use anyhow::Result;
use joule_profiler_core::{JouleProfiler, source::MetricReader};
use source_nvml::config::NvmlConfig;
use source_perf_event::config::PerfConfig;
use source_rapl::{perf::config::RaplPerfConfig, powercap::config::RaplPowercapConfig};
use std::collections::HashMap;

use crate::{CliArgs, RaplBackend};

#[macro_export]
macro_rules! register_sources {
    ($profiler:expr, $config_table:expr, [$($source:ty),* $(,)?]) => {
        $($config_table.register::<$source>($profiler)?;)*
    };
}

pub struct ConfigTable<'a> {
    cli: &'a CliArgs,
    inner: HashMap<String, toml::Value>,
}

impl<'a> ConfigTable<'a> {
    pub fn new(table: toml::Table, cli: &'a CliArgs) -> Self {
        Self {
            cli,
            inner: table.into_iter().collect(),
        }
    }

    pub fn register<R>(&mut self, profiler: &mut JouleProfiler) -> Result<()>
    where
        R: MetricReader + 'static,
        R::Config: CliOverride,
    {
        let (mut config, has_toml) = match self.inner.remove(R::get_id()) {
            Some(v) => (v.try_into().unwrap_or_default(), true),
            None => (R::Config::default(), false),
        };

        let is_activated = has_toml || config.is_enabled(self.cli);
        config.apply_override(self.cli);

        if is_activated {
            profiler.add_source(R::from_config(config)?);
        }

        Ok(())
    }
}

impl<'a> TryFrom<&'a CliArgs> for ConfigTable<'a> {
    type Error = anyhow::Error;

    fn try_from(cli: &'a CliArgs) -> Result<Self, Self::Error> {
        let config_table = if let Some(config_file) = &cli.config_file {
            let content = std::fs::read_to_string(config_file)?;
            let mut value: toml::Table = toml::from_str(&content)?;
            let sources = value
                .remove("sources")
                .unwrap_or(toml::Value::Table(toml::Table::new()))
                .try_into()?;

            ConfigTable::new(sources, cli)
        } else {
            ConfigTable::new(toml::Table::new(), cli)
        };
        Ok(config_table)
    }
}

pub trait CliOverride: Sized + Default {
    fn apply_override(&mut self, _cli: &CliArgs) {}

    fn is_enabled(&self, _cli: &CliArgs) -> bool {
        false
    }
}

impl CliOverride for () {}

impl CliOverride for NvmlConfig {
    fn is_enabled(&self, cli: &CliArgs) -> bool {
        cli.gpu
    }
}

impl CliOverride for PerfConfig {
    fn is_enabled(&self, cli: &CliArgs) -> bool {
        cli.perf
    }
}

impl CliOverride for RaplPowercapConfig {
    fn is_enabled(&self, cli: &CliArgs) -> bool {
        matches!(cli.rapl_backend, RaplBackend::Powercap)
    }
}

impl CliOverride for RaplPerfConfig {
    fn is_enabled(&self, cli: &CliArgs) -> bool {
        matches!(cli.rapl_backend, RaplBackend::Perf)
    }
}
