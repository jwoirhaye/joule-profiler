use thiserror::Error;

#[derive(Debug, Error)]
pub enum DisplayerError {
    #[error("Not implemented for this format")]
    NotImplementedForFormat,

    #[error("I/O error")]
    IoError(
        #[from]
        #[source]
        std::io::Error,
    ),

    #[error("Serialization error")]
    SerializeError(
        #[from]
        #[source]
        serde_json::Error,
    ),
}
