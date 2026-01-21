use std::fmt::Display;

use crate::sources::rapl::error::RaplError;

/// Types of RAPL (Running Average Power Limit) energy/power measurement domains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RaplDomainType {
    /// The entire CPU package.
    Package,

    /// Individual CPU cores.
    Core,

    /// Uncore parts of the CPU (e.g., L3 cache, interconnect).
    Uncore,

    /// DRAM memory.
    Dram,

    /// Platform/system power domain (platform or PSU-level measurements).
    Psys,
}

impl RaplDomainType {
    /// Get the domain name, with its type and socket number
    pub fn to_string_socket(self, socket: u32) -> String {
        format!("{}-{}", self, socket)
    }
}

impl Display for RaplDomainType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let domain_type = match self {
            RaplDomainType::Package => "PACKAGE",
            RaplDomainType::Core => "CORE",
            RaplDomainType::Uncore => "UNCORE",
            RaplDomainType::Dram => "DRAM",
            RaplDomainType::Psys => "PSYS",
        };
        f.write_str(domain_type)
    }
}

impl TryInto<RaplDomainType> for String {
    type Error = RaplError;

    fn try_into(self) -> Result<RaplDomainType, RaplError> {
        let lowercase = self.to_lowercase();
        let domain_type = if lowercase.starts_with("package") {
            RaplDomainType::Package
        } else if lowercase.starts_with("core") {
            RaplDomainType::Core
        } else if lowercase.starts_with("uncore") {
            RaplDomainType::Uncore
        } else if lowercase.starts_with("dram") {
            RaplDomainType::Dram
        } else if lowercase.starts_with("psys") {
            RaplDomainType::Psys
        } else {
            return Err(RaplError::UnknownDomain(lowercase));
        };
        Ok(domain_type)
    }
}
