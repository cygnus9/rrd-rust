//! RRD-related errors.

use std::ffi::{self, CStr, NulError};

use thiserror::Error;

/// Top-level RRD error used for all `librrd` operations.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum RrdError {
    /// A string contained `\0`, and thus could not be converted to a C string
    #[error(transparent)]
    NulError(#[from] NulError),

    /// A path contained non-UTF-8 data
    #[error("Path encoding error")]
    PathEncodingError,

    /// An error from the underlying C librrd library
    #[error("librrd: \"{0}\"")]
    LibRrdError(String),

    /// A miscellaneous error in this library
    #[error("Internal error: {0}")]
    Internal(String),

    /// An [`InvalidArgument`] error
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
}

/// A `Result<T, RrdError>`, a combo used throughout this library
pub type RrdResult<T> = Result<T, RrdError>;

/// Indicates that a Rust wrapper type rejected an invalid argument.
#[derive(Debug, PartialEq, Eq, Error)]
#[error("Invalid argument: {0}")]
pub struct InvalidArgument(pub(crate) &'static str);

impl From<InvalidArgument> for RrdError {
    fn from(value: InvalidArgument) -> Self {
        RrdError::InvalidArgument(value.0.to_string())
    }
}

/// Map `0` to `Ok`, anything else to `Err`
pub(crate) fn return_code_to_result(rc: ffi::c_int) -> RrdResult<()> {
    match rc {
        0 => Ok(()),
        _ => Err(get_rrd_error().unwrap_or_else(|| {
            RrdError::Internal("Unknown error - no librrd error info".to_string())
        })),
    }
}

/// Returns `None` if `rrd_get_error()` return null, otherwise an `RrdError` with the error string..
pub(crate) fn get_rrd_error() -> Option<RrdError> {
    unsafe {
        let p = rrd_sys::rrd_get_error();
        if p.is_null() {
            None
        } else {
            let string = CStr::from_ptr(p).to_string_lossy().into_owned();
            rrd_sys::rrd_clear_error();
            Some(RrdError::LibRrdError(string))
        }
    }
}
