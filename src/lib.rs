pub mod data;
pub mod error;
pub mod util;

pub(crate) mod ops;
use std::ffi::CStr;

pub use ops::create::create;
pub use ops::fetch::fetch;
pub use ops::update::{update, ExtraFlags};

fn get_error() -> String {
    unsafe {
        let p = rrd_sys::rrd_get_error();
        let s = CStr::from_ptr(p);
        s.to_str().unwrap().to_owned()
    }
}
