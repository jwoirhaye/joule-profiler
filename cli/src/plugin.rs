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
    R::Config: CliOverride,
{
    let (config, has_toml) = match self.inner.remove(R::get_id()) {
        Some(v) => (v.try_into()?, true),
        None => (R::Config::default(), false),
    };

    if !R::Config::is_enabled(self.cli, has_toml) {
        return Ok(None);
    }

    Ok(Some(config.apply_override(self.cli)))
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
    fn apply_override(self, cli: &CliArgs) -> Self {
        self
    }

    fn is_enabled(cli: &CliArgs, has_toml: bool) -> bool;
}

impl CliOverride for () {
    fn is_enabled(_cli: &CliArgs, has_toml: bool) -> bool {
        has_toml
    }
}

impl CliOverride for NvmlConfig {
    fn is_enabled(cli: &CliArgs, has_toml: bool) -> bool {
        cli.gpu || has_toml
    }
}

impl CliOverride for PerfConfig {
    fn is_enabled(cli: &CliArgs, has_toml: bool) -> bool {
        cli.perf || has_toml
    }
}

impl CliOverride for RaplPowercapConfig {
    fn is_enabled(cli: &CliArgs, _has_toml: bool) -> bool {
        matches!(cli.rapl_backend, RaplBackend::Powercap)
    }
}

impl CliOverride for RaplPerfConfig {
    fn is_enabled(cli: &CliArgs, _has_toml: bool) -> bool {
        matches!(cli.rapl_backend, RaplBackend::Perf)
    }
}
