use crate::error::RaplError;
use std::fmt::Display;

/// Unique identifier for a domain and socket.
pub type RaplDomainIndex = (RaplDomainType, u32);

/// Types of RAPL (Running Average Power Limit) energy/power measurement domains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RaplDomainType {
    /// Entire processor socket.
    ///
    /// Includes core and uncore components.
    /// Available on all Intel processor since Sandy Bridge generation.
    /// Also known as PKG.
    Package,

    /// CPU cores only.
    ///
    /// Also known as PP0.
    Core,

    /// Integrated graphics device if available.
    ///
    /// Also known as PP1.
    Uncore,

    /// Random access memory attached to the CPU memory controller.
    Dram,

    /// Platform-level power (System on Chip).
    ///
    /// Measures total platform power including Package and additional SoC components.
    Psys,
}

impl RaplDomainType {
    /// Get the domain name, with its type and socket number.
    pub fn to_string_socket(self, socket: u32) -> String {
        match self {
            RaplDomainType::Psys => RaplDomainType::Psys.to_string(),
            domain_type => format!("{}-{}", domain_type, socket),
        }
    }

    /// Converts event to str.
    pub fn to_perf_event(self) -> &'static str {
        match self {
            RaplDomainType::Package => "energy-pkg",
            RaplDomainType::Core => "energy-cores",
            RaplDomainType::Uncore => "energy-gpu",
            RaplDomainType::Dram => "energy-ram",
            RaplDomainType::Psys => "energy-psys",
        }
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
        let name_lower = self.to_lowercase();

        let domain_type = match name_lower.as_str() {
            domain if domain.starts_with("package") => RaplDomainType::Package,
            "energy-pkg" => RaplDomainType::Package,
            "core" | "pp0" | "energy-cores" => RaplDomainType::Core,
            "uncore" | "pp1" | "energy-uncore" | "energy-gpu" => RaplDomainType::Uncore,
            "dram" | "ram" | "energy-ram" => RaplDomainType::Dram,
            "psys" | "platform" | "energy-psys" => RaplDomainType::Psys,
            _ => return Err(RaplError::UnknownDomain(name_lower)),
        };
        Ok(domain_type)
    }
}
