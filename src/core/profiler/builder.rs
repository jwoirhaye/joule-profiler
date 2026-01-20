use crate::{JouleProfiler, config::Config, core::{displayer::Displayer, source::MetricSource}, output::OutputFormat};

#[derive(Default)]
pub struct JouleProfilerBuilder {
    /// The different metric sources
    sources: Vec<Box<dyn MetricSource>>,

    /// Optional displayer for the profiler
    displayer: Option<Box<dyn Displayer>>,

    /// Optional path to the RAPL sysfs interface
    rapl_path: Option<String>,

    /// Format of the output (Terminal, JSON, CSV)
    output_format: OutputFormat,

    /// Optional file to write the output to
    output_file: Option<String>,
}

impl JouleProfilerBuilder {
    pub fn with_source(mut self, source: Box<dyn MetricSource>) -> Self {
        self.sources.push(source);
        self
    }

    pub fn displayer(mut self, displayer: Box<dyn Displayer>) -> Self {
        self.displayer = Some(displayer);
        self
    }

    pub fn rapl_path(mut self, path: String) -> Self {
        self.rapl_path = Some(path);
        self
    }

    pub fn output_format(mut self, output_format: OutputFormat) -> Self {
        self.output_format = output_format;
        self
    }

    pub fn output_file(mut self, output_file: Option<String>) -> Self {
        self.output_file = output_file;
        self
    }

    pub fn build(self) -> JouleProfiler {
        self.into()
    }
}

impl From<JouleProfilerBuilder> for JouleProfiler {
    fn from(builder: JouleProfilerBuilder) -> Self {
        let config = Config { output_file: builder.output_file, output_format: builder.output_format, rapl_path: builder.rapl_path, ..Default::default() };
        Self { displayer: builder.displayer.unwrap_or_default(), config, sources: builder.sources, ..Default::default() }
    }
}