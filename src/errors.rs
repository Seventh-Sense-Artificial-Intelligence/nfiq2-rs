use thiserror::Error;

#[derive(Error, Debug, uniffi::Enum)]
pub enum Nfiq2Error {
    #[error("Null context provided")]
    NullContext,

    #[error("Failed to create NFIQ2 object")]
    CreateFailed,

    #[error("NFIQ2 computation failed with error code: {0}")]
    ComputeFailed(i32),
}
