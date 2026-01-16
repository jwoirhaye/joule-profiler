use enum_dispatch::enum_dispatch;

use crate::core::source::MetricReader;
use crate::sources::rapl::Rapl;

pub mod rapl;

use std::time::Duration;

use anyhow::Result;

use crate::core::{sensor::Sensor, source::SourceResult};

#[enum_dispatch(MetricReader)]
#[derive(Clone, Debug)]
pub enum MetricSource {
    Rapl(Rapl),
}
