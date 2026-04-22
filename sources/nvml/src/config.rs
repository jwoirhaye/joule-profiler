use serde::Deserialize;

#[derive(Default, Deserialize, Debug)]
pub struct NvmlConfig {
    pub gpus_spec: Option<Vec<u32>>,

    #[serde(default)]
    pub exit_on_device_failure: bool,
}
