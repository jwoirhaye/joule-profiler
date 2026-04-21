use anyhow::Result;
use joule_profiler_core::source::{MetricReader, MetricSource};
use source_nvml::config::NvmlConfig;
use source_perf_event::config::PerfConfig;
use source_rapl::{perf::config::RaplPerfConfig, powercap::config::RaplPowercapConfig};
use std::collections::HashMap;

use crate::{CliArgs, RaplBackend};

#[macro_export]
macro_rules! register_sources {
    ($configs:expr, [$($source:ty),* $(,)?]) => {
        $($configs.register::<$source>()?;)*
    };
}

type InitFn = Box<dyn FnOnce() -> Result<Box<dyn MetricSource>>>;

pub struct ConfigTable<'a> {
    cli: &'a CliArgs,
    inner: HashMap<String, toml::Value>,
    init_fns: HashMap<String, InitFn>,
}

impl<'a> ConfigTable<'a> {
    pub fn new(table: toml::Table, cli: &'a CliArgs) -> Self {
        Self {
            cli,
            inner: table.into_iter().collect(),
            init_fns: HashMap::new(),
        }
    }

    pub fn register<R>(&mut self) -> Result<()>
    where
        R: MetricReader + 'static,
        R::Config: CliOverride,
    {
        let (toml_config, has_toml) = match self.inner.remove(R::get_id()) {
            Some(v) => (v.try_into().unwrap_or_default(), true),
            None => (R::Config::default(), false),
        };

        let is_activated = has_toml || toml_config.is_enabled(self.cli);
        let config = toml_config.apply_override(self.cli);

        let init_fn: InitFn = Box::new(move || Ok(R::from_config(config)?.into()));

        if is_activated {
            self.init_fns.insert(R::get_id().to_string(), init_fn);
        }

        Ok(())
    }

    pub fn build_sources(self) -> Result<Vec<Box<dyn MetricSource>>> {
        let mut sources = Vec::new();
        for (_id, init_fn) in self.init_fns {
            sources.push(init_fn()?);
        }
        Ok(sources)
    }
}

pub trait CliOverride: Sized + Default {
    fn apply_override(self, _cli: &CliArgs) -> Self {
        self
    }

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
