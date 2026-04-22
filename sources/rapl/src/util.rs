use std::collections::HashSet;

use serde::{Deserialize, Deserializer};

use crate::Result;

/// Checks if the operating system is Linux.
#[allow(clippy::unnecessary_wraps)]
pub fn check_os() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    {
        let os = std::env::consts::OS;
        Err(RaplError::UnsupportedOS(os.to_string()).into())
    }
}

#[derive(Debug, Default)]
pub enum SocketSelector {
    #[default]
    All,
    List(HashSet<u32>),
}

impl<'de> Deserialize<'de> for SocketSelector {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        if s.trim() == "*" {
            return Ok(SocketSelector::All);
        }
        let ids = s
            .split(',')
            .map(|part| part.trim().parse::<u32>().map_err(serde::de::Error::custom))
            .collect::<std::result::Result<HashSet<_>, _>>()?;
        Ok(SocketSelector::List(ids))
    }
}
