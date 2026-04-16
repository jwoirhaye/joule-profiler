use joule_profiler_core::{transformer::MetricTransformer, types::Metrics};

pub struct RaplPerProcessAttributionTransformer;

impl MetricTransformer for RaplPerProcessAttributionTransformer {
    fn transform(&self, metrics: &mut Metrics) {
        let global_cpu_cycles = if let Some(global_cpu_cycles) = metrics.iter().find(|m| m.name == "GLOBAL_CPU_CYCLES") {
            global_cpu_cycles.value
        } else {
            return;
        };
        
        let cpu_cycles = if let Some(cpu_cycles) = metrics.iter().find(|m| m.name == "CPU_CYCLES") {
            cpu_cycles.value
        } else {
            return;
        };
        
        metrics.retain(|m| m.name != "GLOBAL_CPU_CYCLES");
        
        let ratio = cpu_cycles as f64 / global_cpu_cycles as f64;
        println!("ratio {}", ratio);
        
        for metric in metrics.iter_mut().filter(|m| m.source.starts_with("RAPL")) {
            metric.value = (metric.value as f64 * ratio) as u64
        }
    }
}
