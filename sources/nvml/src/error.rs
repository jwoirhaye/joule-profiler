use thiserror::Error;

#[derive(Debug, Error)]
pub enum NvmlError {
    #[error("Unknow metric \"{0}\" found in old snapshot")]
    UnknownMetricError(String),

    #[error(
        "No driver found or loaded to access NVML, check whether you have an Nvidia GPU or not"
    )]
    NoDriverLoaded,

    #[error("Insufficient permissions to access NVML. Try running with sudo")]
    NoPermission,

    #[error("NVML error: {0}")]
    NvmlError(
        #[from]
        #[source]
        nvml_wrapper::error::NvmlError,
    ),

    #[error("Not enough measures to compute GPU energy counters differences")]
    NotEnoughSamples,
}
