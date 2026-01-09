use anyhow::Result;

use crate::source::metric::Metrics;

pub mod metric;
pub mod rapl;

pub trait MetricReader {
    fn measure(&mut self) -> Result<()>;
    fn retrieve(&mut self) -> Result<Vec<Metrics>>;
}

pub struct MetricSource {
    pub ctx: *mut (),
    pub measure: fn(*mut ()) -> Result<()>,
    pub retrieve: fn(*mut ()) -> Result<Vec<Metrics>>,
}

impl MetricSource {
    pub fn measure(&self) -> Result<()> {
        (self.measure)(self.ctx)
    }

    pub fn retrieve(&self) -> Result<Vec<Metrics>> {
        (self.retrieve)(self.ctx)
    }
}

impl<T: MetricReader> From<T> for MetricSource {
    // fn from(mut value: T) -> Self {
    fn from(value: T) -> Self {
        fn measure<T: MetricReader>(ctx: *mut ()) -> Result<()> {
            let ptr = ctx as *mut T;
            unsafe { (*ptr).measure() }
        }

        fn retrieve<T: MetricReader>(ctx: *mut ()) -> Result<Vec<Metrics>> {
            let ptr = ctx as *mut T;
            unsafe { (*ptr).retrieve() }
        }

        // let ptr = &raw mut value as *mut ();
        let boxed = Box::new(value);

        Self {
            ctx: Box::into_raw(boxed) as *mut (),
            // ctx: ptr,
            measure: measure::<T>,
            retrieve: retrieve::<T>,
        }
    }
}
