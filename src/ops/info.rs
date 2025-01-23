//! Get info about an RRD.

use crate::{
    error::{get_rrd_error, RrdError, RrdResult},
    util::path_to_str,
};
use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    path::Path,
};

/// Returns a map of metadata about the RRD at `filename`.
///
/// The contents vary based on RRD structure, but generally provide info about each data source and
/// RRA.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdinfo.en.html>.
pub fn info(filename: &Path) -> RrdResult<HashMap<String, InfoValue>> {
    let filename = CString::new(path_to_str(filename)?)?;

    let result_ptr = unsafe { rrd_sys::rrd_info_r(filename.as_ptr()) };
    if result_ptr.is_null() {
        return Err(get_rrd_error().unwrap_or_else(|| {
            RrdError::Internal("No info data, but no librrd error".to_string())
        }));
    }

    Ok(build_info_map(result_ptr))
}

/// Value in the map returned from [`info()`], and other places that use the same info map.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
#[allow(missing_docs)]
pub enum InfoValue {
    Value(f64),
    Count(u64),
    String(String),
    Int(i32),
    Blob(Vec<u8>),
}

impl InfoValue {
    /// Returns `Some` if this is an `InfoValue::Value`, `None` otherwise
    pub fn into_value(self) -> Option<f64> {
        match self {
            InfoValue::Value(v) => Some(v),
            InfoValue::Count(_) | InfoValue::String(_) | InfoValue::Int(_) | InfoValue::Blob(_) => {
                None
            }
        }
    }

    /// Returns `Some` if this is an `InfoValue::Count`, `None` otherwise
    pub fn into_count(self) -> Option<u64> {
        match self {
            InfoValue::Count(c) => Some(c),
            InfoValue::Value(_) | InfoValue::String(_) | InfoValue::Int(_) | InfoValue::Blob(_) => {
                None
            }
        }
    }

    /// Returns `Some` if this is an `InfoValue::String`, `None` otherwise
    pub fn into_string(self) -> Option<String> {
        match self {
            InfoValue::String(s) => Some(s),
            InfoValue::Value(_) | InfoValue::Count(_) | InfoValue::Int(_) | InfoValue::Blob(_) => {
                None
            }
        }
    }

    /// Returns `Some` if this is an `InfoValue::Int`, `None` otherwise
    pub fn into_int(self) -> Option<i32> {
        match self {
            InfoValue::Int(i) => Some(i),
            InfoValue::Value(_)
            | InfoValue::Count(_)
            | InfoValue::String(_)
            | InfoValue::Blob(_) => None,
        }
    }

    /// Returns `Some` if this is an `InfoValue::Blob`, `None` otherwise
    pub fn into_blob(self) -> Option<Vec<u8>> {
        match self {
            InfoValue::Blob(b) => Some(b),
            InfoValue::Value(_)
            | InfoValue::Count(_)
            | InfoValue::String(_)
            | InfoValue::Int(_) => None,
        }
    }
}

impl From<f64> for InfoValue {
    fn from(value: f64) -> Self {
        Self::Value(value)
    }
}
impl From<u64> for InfoValue {
    fn from(value: u64) -> Self {
        Self::Count(value)
    }
}
impl From<String> for InfoValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}
impl From<&str> for InfoValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}
impl From<i32> for InfoValue {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}
impl From<Vec<u8>> for InfoValue {
    fn from(value: Vec<u8>) -> Self {
        Self::Blob(value)
    }
}

/// Must only be called on a non-null pointer.
///
/// # Panics
///
/// Will panic if `info` is null.
pub(crate) fn build_info_map(info: *mut rrd_sys::rrd_info_t) -> HashMap<String, InfoValue> {
    assert!(!info.is_null());

    let mut map = HashMap::new();
    let mut current = info;
    while !current.is_null() {
        let key_cstr = unsafe { CStr::from_ptr((*current).key) };
        let key = key_cstr.to_string_lossy().into_owned();

        let value = match *unsafe { &(*current).type_ } {
            rrd_sys::rrd_info_type_RD_I_VAL => (unsafe { (*current).value.u_val }).into(),
            rrd_sys::rrd_info_type_RD_I_CNT => {
                // on windows, ffi::c_ulong is u32
                #[allow(clippy::useless_conversion)]
                u64::from(unsafe { (*current).value.u_cnt }).into()
            }
            rrd_sys::rrd_info_type_RD_I_STR => {
                let str_cstr = unsafe { CStr::from_ptr((*current).value.u_str) };
                // Realistically people will probably just use `to_string_lossy` anyway,
                // so not exposing the Result seems suitable.
                str_cstr.to_string_lossy().into_owned().into()
            }
            rrd_sys::rrd_info_type_RD_I_INT => (unsafe { (*current).value.u_int }).into(),
            rrd_sys::rrd_info_type_RD_I_BLO => {
                let slice = unsafe {
                    let blob = (*current).value.u_blo;
                    std::slice::from_raw_parts(
                        blob.ptr.cast_const(),
                        blob.size.try_into().expect("Implausibly huge blob"),
                    )
                };

                slice.to_vec().into()
            }
            t => {
                panic!("Unexpected info type {t} - version mismatch, or memory corruption?")
            }
        };

        map.insert(key, value);

        current = unsafe { (*current).next };
    }

    unsafe { rrd_sys::rrd_info_free(info) }

    map
}
