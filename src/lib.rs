pub mod data;
pub mod error;
pub mod util;

pub mod ops;

pub use ops::create::create;
pub use ops::fetch::fetch;
pub use ops::graph::graph;
pub use ops::info::info;

// since it's in the public API, expose it for users
pub use chrono;

pub type Timestamp = chrono::DateTime<chrono::Utc>;

pub(crate) trait TimestampExt {
    fn as_time_t(&self) -> rrd_sys::time_t;
}

impl TimestampExt for Timestamp {
    fn as_time_t(&self) -> rrd_sys::time_t {
        self.timestamp()
    }
}

/// How to aggregate primary data points in a RRA
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
