//! Rust API for [`librrd`](https://oss.oetiker.ch/rrdtool/index.en.html).
//!
//! The Rust wrappers for supported `librrd` functions are in [`ops`], e.g. [`ops::create`].
//!
//! See the `/examples` directory or `tests/tutorial.rs` for detailed examples. The latter is a
//! recreation of <https://oss.oetiker.ch/rrdtool/tut/rrdtutorial.en.html>, which uses the CLI
//! tools, with this library.
//!
//! # Logging
//!
//! If unexpected behavior is observed, it can be helpful to see exactly what paramters are being
//! provided to the underlying `librrd` functions. For operations that do any level of mapping of
//! their input into `librrd` input, the [`log`](https://crates.io/crates/log) crate is used at
//! `debug` level, so log output can be enabled with `RUST_LOG=rrd=debug` (if using `env_logger`)
//! or other means of configuring `log`.

#![deny(missing_docs)]

use crate::error::{RrdError, RrdResult};
use std::time;

// TODO get confirmation from upstream about librrd thread safety
pub mod data;
pub mod error;
pub mod ops;
pub mod util;

/// The point in time associated with a data point.
pub type Timestamp = std::time::SystemTime;

/// Internal extensions for [`Timestamp`]
pub(crate) trait TimestampExt {
    /// Returns the timestamp as seconds since epoch.
    fn try_as_time_t(&self) -> RrdResult<rrd_sys::time_t>;

    /// Creates a timestamp from seconds since epoch.
    fn try_from_time_t(time_t: rrd_sys::time_t) -> RrdResult<Self>
    where
        Self: Sized;
}

impl TimestampExt for Timestamp {
    fn try_as_time_t(&self) -> RrdResult<rrd_sys::time_t> {
        let duration = self.duration_since(std::time::UNIX_EPOCH).map_err(|_| {
            RrdError::InvalidArgument("timestamp must be after UNIX_EPOCH".to_string())
        })?;
        i64::try_from(duration.as_secs())
            .map_err(|_| RrdError::InvalidArgument("timestamp is too large for librrd".to_string()))
    }

    fn try_from_time_t(time_t: rrd_sys::time_t) -> RrdResult<Self> {
        let secs = u64::try_from(time_t).map_err(|_| {
            RrdError::Internal(format!("librrd returned negative timestamp {time_t}"))
        })?;
        Ok(time::UNIX_EPOCH + time::Duration::from_secs(secs))
    }
}

/// How to aggregate primary data points in a RRA.
///
/// See [`ops::create::Archive`] and [`ops::graph::elements::Def`].
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsolidationFn {
    Avg,
    Min,
    Max,
    Last,
}

impl ConsolidationFn {
    pub(crate) fn as_arg_str(&self) -> &str {
        match self {
            ConsolidationFn::Avg => "AVERAGE",
            ConsolidationFn::Min => "MIN",
            ConsolidationFn::Max => "MAX",
            ConsolidationFn::Last => "LAST",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_before_epoch_is_invalid_argument() {
        let timestamp = time::UNIX_EPOCH - time::Duration::from_secs(1);

        assert_eq!(
            Err(RrdError::InvalidArgument(
                "timestamp must be after UNIX_EPOCH".to_string()
            )),
            timestamp.try_as_time_t()
        );
    }

    #[test]
    fn negative_time_t_is_internal_error() {
        assert_eq!(
            Err(RrdError::Internal(
                "librrd returned negative timestamp -1".to_string()
            )),
            Timestamp::try_from_time_t(-1)
        );
    }
}
