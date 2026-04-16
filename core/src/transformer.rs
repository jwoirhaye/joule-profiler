use crate::types::{Metrics, ProfilerResults};

pub trait MetricTransformer: 'static {
    fn transform(&self, metrics: &mut Metrics);
}

pub(crate) struct MetricTransformerWrapper {
    order: i16,
    transformer: Box<dyn MetricTransformer>,
}

#[derive(Default)]
pub(crate) struct GlobalMetricTransformer {
    transformers: Vec<MetricTransformerWrapper>,
}

impl GlobalMetricTransformer {
    pub fn add_transformer<T>(&mut self, transformer: T, order: i16)
    where
        T: MetricTransformer,
    {
        let wrapper = MetricTransformerWrapper {
            order,
            transformer: Box::new(transformer),
        };
        let index = self.transformers.partition_point(|w| w.order >= order);
        self.transformers.insert(index, wrapper);
    }

    pub fn transform(&self, results: &mut ProfilerResults) {
        for transformer in self.transformers.iter().map(|w| w.transformer.as_ref()) {
            for result in &mut results.phases {
                transformer.transform(&mut result.metrics);
            }
        }
    }
}