use std::ffi::CStr;

/// Returns the version of `librrd` in use.
pub fn librrd_version() -> String {
    (unsafe { CStr::from_ptr(rrd_sys::rrd_strversion()) })
        .to_string_lossy()
        .into_owned()
}
