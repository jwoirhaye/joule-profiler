use thiserror::Error;

/// Errors that can occur when using the NVML source.
#[derive(Debug, Error)]
pub enum NvmlError {
    /// A device index present in the old snapshot was not found in the new one.
    #[error("Unknow metric \"{0}\" found in old snapshot")]
    UnknownMetricError(String),

    /// NVML could not find or load the NVIDIA driver.
    #[error(
        "No driver found or loaded to access NVML, check whether you have an Nvidia GPU or not"
    )]
    NoDriverLoaded,

    /// The process lacks the required permissions to access NVML.
    #[error("Insufficient permissions to access NVML. Try running with sudo")]
    NoPermission,

    /// Error propagated from the underlying `nvml_wrapper` library.
    #[error("NVML error: {0}")]
    NvmlError(
        #[from]
        #[source]
        nvml_wrapper::error::NvmlError,
    ),

    /// Not enough snapshots have been taken to compute an energy delta.
    #[error("Not enough measures to compute GPU energy counters differences")]
    NotEnoughSamples,
}
