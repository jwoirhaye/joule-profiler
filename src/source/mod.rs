use anyhow::Result;

use crate::{config::OutputFormat, source::metric::Snapshot};

pub mod metric;
pub mod rapl;

// pub trait MetricReader {
//     fn measure(&mut self) -> Result<()>;
//     fn retrieve(&mut self) -> Result<Vec<Snapshot>>;
//     fn print_source(&self, format: OutputFormat) -> Result<()>;
// }

// pub struct MetricSource {
//     pub ctx: *mut (),
//     pub measure: fn(*mut ()) -> Result<()>,
//     pub retrieve: fn(*mut ()) -> Result<Vec<Snapshot>>,
//     pub print_source: fn(*mut (), OutputFormat) -> Result<()>
// }

// impl MetricSource {
//     pub fn measure(&self) -> Result<()> {
//         (self.measure)(self.ctx)
//     }

//     pub fn retrieve(&self) -> Result<Vec<Snapshot>> {
//         (self.retrieve)(self.ctx)
//     }

//     pub fn print_source(&self, format: OutputFormat) -> Result<()> {
//         (self.print_source)(self.ctx, format)
//     }
// }

// impl<T: MetricReader> From<T> for MetricSource {
//     fn from(value: T) -> Self {
//         fn measure<T: MetricReader>(ctx: *mut ()) -> Result<()> {
//             let ptr = ctx as *mut T;
//             unsafe { (*ptr).measure() }
//         }

//         fn retrieve<T: MetricReader>(ctx: *mut ()) -> Result<Vec<Snapshot>> {
//             let ptr = ctx as *mut T;
//             unsafe { (*ptr).retrieve() }
//         }

//         fn print_source<T: MetricReader>(ctx: *mut (), format: OutputFormat) -> Result<()> {
//             let ptr = ctx as *mut T;
//             unsafe { (*ptr).print_source(format) }
//         }

//         let boxed = Box::new(value);

//         Self {
//             ctx: Box::into_raw(boxed) as *mut (),
//             measure: measure::<T>,
//             retrieve: retrieve::<T>,
//             print_source: print_source::<T>,
//         }
//     }
// }

pub trait MetricReader {
    fn measure(&mut self) -> Result<()>;
    fn retrieve(&mut self) -> Result<Vec<Snapshot>>;
    fn print_source(&self, format: OutputFormat) -> Result<()>;
}

pub type MetricSource = Box<dyn MetricReader>;
