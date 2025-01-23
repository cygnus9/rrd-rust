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

// TODO get confirmation from upstream about librrd thread safety
pub mod data;
pub mod error;
pub mod ops;
pub mod util;

// `chrono::DateTime` and `chrono::Utc` are used for timestamps, so this is provided to allow
// easy access without a separate `chrono` dependency.
pub use chrono;

/// The point in time associated with a data point.
pub type Timestamp = chrono::DateTime<chrono::Utc>;

/// Internal extensions for [`Timestamp`]
pub(crate) trait TimestampExt {
    /// Returns the timestamp as seconds since epoch.
    fn as_time_t(&self) -> rrd_sys::time_t;
}

impl TimestampExt for Timestamp {
    fn as_time_t(&self) -> rrd_sys::time_t {
        self.timestamp()
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
