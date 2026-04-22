use serde::Deserialize;

#[derive(Default, Deserialize, Debug)]
pub struct RaplPerfConfig {
    pub sockets_spec: Option<Vec<u32>>,
}
