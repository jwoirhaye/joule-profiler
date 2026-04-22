use std::time::Duration;

use serde::Deserialize;

#[derive(Default, Deserialize, Debug)]
pub struct RaplPowercapConfig {
    pub sockets_spec: Option<Vec<u32>>,

    #[serde(default, with = "humantime_serde::option")]
    pub poll_interval: Option<Duration>,

    pub rapl_path: Option<String>,
}
