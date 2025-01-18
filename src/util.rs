use rrd_sys::rrd_char;
use std::{ffi::CString, fmt, path::Path, ptr, result::Result};

use crate::error::{RrdError, RrdResult};

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

pub(crate) struct MaybeNullTerminatedArrayOfStrings<const IS_NULL_TERMINATED: bool> {
    /// Keep the strings so they can be dropped
    cstrings: Vec<CString>,
    pointers: Vec<*const rrd_char>,
}

impl<const IS_NULL_TERMINATED: bool> MaybeNullTerminatedArrayOfStrings<IS_NULL_TERMINATED> {
    pub fn new<T, U>(strings: T) -> RrdResult<Self>
    where
        T: IntoIterator<Item = U>,
        U: Into<String>,
    {
        let cstrings = strings
            .into_iter()
            .map(|s| CString::new(s.into()))
            .collect::<Result<Vec<_>, _>>()?;
        let mut pointers = cstrings.iter().map(|cs| cs.as_ptr()).collect::<Vec<_>>();
        if IS_NULL_TERMINATED {
            pointers.push(ptr::null());
        }
        Ok(Self { cstrings, pointers })
    }

    pub fn as_ptr(&self) -> *mut *const rrd_char {
        if self.is_empty() {
            ptr::null_mut()
        } else {
            self.pointers.as_ptr().cast_mut()
        }
    }

    pub fn len(&self) -> usize {
        self.cstrings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cstrings.is_empty()
    }
}

impl<const T: bool> fmt::Debug for MaybeNullTerminatedArrayOfStrings<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.cstrings.iter()).finish()
    }
}

/// Represents an array of pointers to nul-terminated strings.
///
/// These should be used with C APIs that also take a length parameter.
pub(crate) type ArrayOfStrings = MaybeNullTerminatedArrayOfStrings<false>;

/// Represents a nul-terminated array of pointers to nul-terminated strings.
///
/// These should be used with C APIs that don't take a length parameter but
/// expect the last pointer in the array to be null.
pub(crate) type NullTerminatedArrayOfStrings = MaybeNullTerminatedArrayOfStrings<true>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr::null_mut;

    #[test]
    fn array_all_ptrs_non_null() {
        let array = ArrayOfStrings::new(["one", "two"]).unwrap();
        assert_eq!(array.len(), 2);
        assert!(!array.is_empty());
        assert_ne!(array.as_ptr(), null_mut());

        unsafe {
            assert_ne!(*(array.as_ptr().add(0)), null_mut());
            assert_ne!(*(array.as_ptr().add(1)), null_mut());
        }
    }

    #[test]
    fn array_empty_is_null() {
        let source: &[String] = &[];
        let array = ArrayOfStrings::new(source).unwrap();
        assert_eq!(array.len(), 0);
        assert!(array.is_empty());
        assert_eq!(array.as_ptr(), null_mut());
    }

    #[test]
    fn null_terminated_array_all_ptrs_non_null() {
        let array = NullTerminatedArrayOfStrings::new(["one", "two"]).unwrap();
        assert_eq!(array.len(), 2);
        assert!(!array.is_empty());
        assert_ne!(array.as_ptr(), null_mut());

        unsafe {
            assert_ne!(*(array.as_ptr().add(0)), null_mut());
            assert_ne!(*(array.as_ptr().add(1)), null_mut());
            assert_eq!(*(array.as_ptr().add(2)), null_mut());
        }
    }

    #[test]
    fn null_terminated_array_empty_is_null() {
        let source: &[String] = &[];
        let array = NullTerminatedArrayOfStrings::new(source).unwrap();
        assert_eq!(array.len(), 0);
        assert!(array.is_empty());
        assert_eq!(array.as_ptr(), null_mut());
    }
}
