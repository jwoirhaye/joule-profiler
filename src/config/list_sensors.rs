use crate::{
    core::displayer::ListSensorsDisplayer,
    output::{OutputFormat, terminal::TerminalOutput},
};

#[derive(Debug, Clone)]
pub struct ListSensorsConfig {
    pub output_format: OutputFormat,
    pub rapl_path: Option<String>,
}

impl TryFrom<&ListSensorsConfig> for Box<dyn ListSensorsDisplayer> {
    type Error = anyhow::Error;

    fn try_from(value: &ListSensorsConfig) -> Result<Self, Self::Error> {
        match value.output_format {
            OutputFormat::Terminal => Ok(Box::new(TerminalOutput)),
            _ => anyhow::bail!(
                "Unsupported output type for list sensors command: {}",
                value.output_format
            ),
        }
    }
}
