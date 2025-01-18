pub mod data;
pub mod error;
pub mod util;

pub mod ops;

pub use ops::create::create;
pub use ops::fetch::fetch;
pub use ops::graph::graph;
pub use ops::info::info;
pub use ops::update::{update, ExtraFlags};

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
