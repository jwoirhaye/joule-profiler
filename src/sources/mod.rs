//! Metric sources implementations.
//!
//! This module contains implementations of metric sources that can be
//! plugged into the Joule Profiler framework. Each source provides
//! a concrete implementation of the [`MetricReader`](`crate::reader::MetricReader`) trait.
//!
//! Currently available sources:
//! - [`Rapl`] — Collects energy metrics from Intel RAPL domains.
//!
//! # Usage
//!
//! To add a new source, implement the [`MetricReader`](`crate::reader::MetricReader`) trait and expose it
//! through this module. For example:
//! You also need to provide an error type and a snapshot type which require some trait bounds,
//! see [`crate::reader`] for further explanation.

pub(crate) mod rapl;
pub use rapl::Rapl;
