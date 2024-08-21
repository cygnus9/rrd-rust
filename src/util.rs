use rrd_sys::rrd_char;
use std::ffi::CString;
use std::path::Path;
use std::ptr::null_mut;
use std::result::Result;
use std::time::{Duration, SystemTime, SystemTimeError};

use crate::error::{RrdError, RrdResult};

/// Convert a `SystemTime` to `time_t` (a.k.a. seconds since unix epoch)
///
/// # Examples
/// ```
/// use std::time::SystemTime;
/// use rrd::util::to_unix_time;
///
/// let now = SystemTime::now();
/// assert!(to_unix_time(now).unwrap() > 0);
/// ```
pub fn to_unix_time(ts: SystemTime) -> Result<rrd_sys::time_t, SystemTimeError> {
    ts.duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs() as rrd_sys::time_t)
}

/// Convert a `time_t` (a.k.a. seconds since epoch) to a `SystemTime`
///
/// # Examples
/// ```
/// use std::ptr::null_mut;
/// use std::time::SystemTime;
/// use libc::time;
/// use rrd::util::from_unix_time;
///
/// let now = unsafe { time(null_mut()) };
/// assert!(from_unix_time(now) > SystemTime::UNIX_EPOCH);
/// ```
pub fn from_unix_time(ts: rrd_sys::time_t) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(ts as u64)
}

/// Conveniently convert a `Path` to a `&str`
///
/// # Examples
/// ```
/// use std::path::Path;
/// use rrd::util::path_to_str;
///
/// let path = Path::new("/some/path");
/// assert_eq!(path_to_str(path).unwrap(), "/some/path");
/// ```
///
/// Paths with invalid UTF-8 will fail though
/// ```
/// #[cfg(unix)] {
///     use std::ffi::OsStr;
///     use std::os::unix::ffi::OsStrExt;
///     use std::path::Path;
///     use rrd::util::path_to_str;
///     
///     let source = [0x66, 0x6f, 0x80, 0x6f];
///     let os_str = OsStr::from_bytes(&source[..]);
///     let path = Path::new(os_str);
///     assert!(path_to_str(path).is_err());
/// }
/// #[cfg(windows)] {
///     use std::ffi::OsString;
///     use std::os::windows::prelude::*;
///     use std::path::Path;
///     use rrd::util::path_to_str;
///     
///     let source = [0x0066, 0x006f, 0xD800, 0x006f];
///     let os_string = OsString::from_wide(&source[..]);
///     let os_str = os_string.as_os_str();
///     let path = Path::new(os_str);
///     assert!(path_to_str(path).is_err());
/// }
/// ```
pub fn path_to_str(path: &Path) -> Result<&str, RrdError> {
    path.to_str().ok_or(RrdError::PathEncodingError)
}

pub struct MaybeNullTerminatedArrayOfStrings<const IS_NULL_TERMINATED: bool> {
    pointers: Vec<*mut rrd_char>,
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

    pub fn as_ptr(&self) -> *mut *const rrd_char {
        if self.is_empty() {
            null_mut()
        } else {
            self.pointers.as_ptr() as *mut *const rrd_char
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

/// Represents an array of pointers to nul-terminated strings.
///
/// These should be used with C APIs that also take a length parameter.
///
/// # Examples
/// ```
/// use std::ptr::null_mut;
/// use rrd::util::ArrayOfStrings;
///
/// let array = ArrayOfStrings::new(["one", "two"]).unwrap();
/// assert_eq!(array.len(), 2);
/// assert!(!array.is_empty());
/// assert_ne!(array.as_ptr(), null_mut());
///
/// unsafe {
///     assert_ne!(*(array.as_ptr().add(0)), null_mut());
///     assert_ne!(*(array.as_ptr().add(1)), null_mut());
/// }
/// ```
///
/// An empty array returns `null` from `as_ptr()`.
/// ```
/// use std::ptr::null_mut;
/// use rrd::util::ArrayOfStrings;
///
/// let source: &[&str] = &[];
/// let array = ArrayOfStrings::new(source).unwrap();
/// assert_eq!(array.len(), 0);
/// assert!(array.is_empty());
/// assert_eq!(array.as_ptr(), null_mut());
/// ```
pub type ArrayOfStrings = MaybeNullTerminatedArrayOfStrings<false>;

/// Represents a nul-terminated array of pointers to nul-terminated strings.
///
/// These should be used with C APIs that don't take a length parameter but
/// expect the last pointer in the array to be null.
///
/// # Examples
/// ```
/// use std::ptr::null_mut;
/// use rrd::util::NullTerminatedArrayOfStrings;
///
/// let array = NullTerminatedArrayOfStrings::new(["one", "two"]).unwrap();
/// assert_eq!(array.len(), 2);
/// assert!(!array.is_empty());
/// assert_ne!(array.as_ptr(), null_mut());
///
/// unsafe {
///     assert_ne!(*(array.as_ptr().add(0)), null_mut());
///     assert_ne!(*(array.as_ptr().add(1)), null_mut());
///     assert_eq!(*(array.as_ptr().add(2)), null_mut());
/// }
/// ```
///
/// An empty array returns `null` from `as_ptr()`.
/// ```
/// use std::ptr::null_mut;
/// use rrd::util::NullTerminatedArrayOfStrings;
///
/// let source: &[&str] = &[];
/// let array = NullTerminatedArrayOfStrings::new(source).unwrap();
/// assert_eq!(array.len(), 0);
/// assert!(array.is_empty());
/// assert_eq!(array.as_ptr(), null_mut());
/// ```
pub type NullTerminatedArrayOfStrings = MaybeNullTerminatedArrayOfStrings<true>;
