use crate::types::Metrics;

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
}

impl<'a> IntoIterator for &'a GlobalMetricTransformer {
    type Item = &'a dyn MetricTransformer;
    type IntoIter = std::iter::Map<
        std::slice::Iter<'a, MetricTransformerWrapper>,
        fn(&'a MetricTransformerWrapper) -> &'a dyn MetricTransformer,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.transformers
            .iter()
            .map(|wrapper| wrapper.transformer.as_ref())
    }
}
