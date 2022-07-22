use std::ffi::NulError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RrdError {
    #[error(transparent)]
    NulError(#[from] NulError),

    #[error("Path encoding error")]
    PathEncodingError,

    #[error("Error from librrd: \"{0}\"")]
    LibRrdError(String),
}

pub type RrdResult<T> = Result<T, RrdError>;
