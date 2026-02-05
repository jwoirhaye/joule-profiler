use thiserror::Error;

#[derive(Debug, Error)]
pub enum NvmlError {
    #[error("Unknow metric \"{0}\" found in old snapshot")]
    UnknownMetricError(String),

    #[error("No driver found or loaded to access NVML")]
    NoDriverLoaded,

    #[error("Insufficient permissions to access NVML. Try running with sudo")]
    NoPermission,

    #[error("Nvml error")]
    NvmlError(
        #[from]
        #[source]
        nvml_wrapper::error::NvmlError,
    ),
}
