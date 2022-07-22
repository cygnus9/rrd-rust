use std::ffi::CString;
use std::path::Path;
use std::ptr::{null, null_mut};
use std::result::Result;
use std::time::{SystemTime, SystemTimeError};

use crate::error::{RrdError, RrdResult};
use crate::sys;

pub fn to_unix_time(ts: &SystemTime) -> Result<sys::c_time_t, SystemTimeError> {
    ts.duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs() as sys::c_time_t)
}

pub fn path_to_str(path: &Path) -> Result<&str, RrdError> {
    path.to_str().ok_or_else(|| RrdError::PathEncodingError)
}

pub struct MaybeNullTerminatedArrayOfStrings<const IS_NULL_TERMINATED: bool> {
    pointers: Vec<*mut sys::c_char>,
}

impl<const IS_NULL_TERMINATED: bool> Drop
    for MaybeNullTerminatedArrayOfStrings<IS_NULL_TERMINATED>
{
    fn drop(&mut self) {
        for p in self.pointers.iter().take_while(|p| !p.is_null()) {
            unsafe {
                let _ = CString::from_raw(*p);
            }
        }
    }
}

impl<const IS_NULL_TERMINATED: bool> MaybeNullTerminatedArrayOfStrings<IS_NULL_TERMINATED> {
    pub fn new<T, U>(strings: T) -> RrdResult<Self>
    where
        T: IntoIterator<Item = U>,
        U: AsRef<str>,
    {
        let mut pointers = strings
            .into_iter()
            .map(|s| CString::new(s.as_ref()))
            .map(|r| r.map(|s| s.into_raw()))
            .collect::<Result<Vec<_>, _>>()?;
        if IS_NULL_TERMINATED {
            pointers.push(null_mut());
        }
        Ok(Self { pointers })
    }

    pub fn as_ptr(&self) -> *const *const sys::c_char {
        if self.is_empty() {
            null()
        } else {
            self.pointers.as_ptr() as *const *const sys::c_char
        }
    }

    pub fn len(&self) -> usize {
        if IS_NULL_TERMINATED {
            self.pointers.len() - 1
        } else {
            self.pointers.len()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub type ArrayOfStrings = MaybeNullTerminatedArrayOfStrings<false>;
pub type NullTerminatedArrayOfStrings = MaybeNullTerminatedArrayOfStrings<true>;
