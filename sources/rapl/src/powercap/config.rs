use std::time::Duration;

use serde::Deserialize;

use crate::util::SocketSelector;

#[derive(Default, Deserialize, Debug)]
pub struct RaplPowercapConfig {
    #[serde(default)]
    pub target_sockets: SocketSelector,

    #[serde(default, with = "humantime_serde::option")]
    pub poll_interval: Option<Duration>,

    pub rapl_path: Option<String>,
}
