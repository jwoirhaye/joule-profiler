use serde::Deserialize;

use crate::util::SocketSelector;

#[derive(Default, Deserialize, Debug)]
pub struct RaplPerfConfig {
    #[serde(default)]
    pub target_sockets: SocketSelector,
}
