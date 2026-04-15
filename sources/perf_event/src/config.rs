use serde::Deserialize;

use crate::event::Event;

#[derive(Debug, Default, Deserialize)]
pub struct PerfConfig {
    pub events: Option<Vec<Event>>,
}
