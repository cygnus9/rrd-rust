//! Get the `librrd` version

use std::ffi::CStr;

/// Returns the version of `librrd` this library is linked to, e.g. `"1.9.0"`.
pub fn librrd_version() -> String {
    (unsafe { CStr::from_ptr(rrd_sys::rrd_strversion()) })
        .to_string_lossy()
        .into_owned()
}
