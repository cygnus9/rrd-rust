//! Fetch data from an RRD.

use crate::{
    data::Data,
    error::{return_code_to_result, RrdResult},
    util::path_to_str,
    ConsolidationFn, Timestamp, TimestampExt,
};
use rrd_sys::{rrd_double, rrd_void};
use std::{
    ffi::{CStr, CString},
    fmt,
    ops::Deref,
    path::Path,
    ptr::null_mut,
    slice,
    time::Duration,
};

/// Fetch data from `filename` between `start` and `end`, consolidated with `cf`.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdfetch.en.html>.
pub fn fetch(
    filename: &Path,
    cf: ConsolidationFn,
    start: Timestamp,
    end: Timestamp,
    resolution: Duration,
) -> RrdResult<Data<Array>> {
    // in
    let filename = CString::new(path_to_str(filename)?)?;
    let cf = CString::new(cf.as_arg_str())?;

    // in/out - clobber var names to avoid accidentally using original input values
    let mut start = start.as_time_t();
    let mut end = end.as_time_t();
    // windows c_ulong is u32
    #[allow(clippy::useless_conversion)]
    let mut resolution = resolution
        .as_secs()
        .try_into()
        .expect("Implausibly long resolution");

    // out
    let mut ds_count = 0;
    let mut ds_names = null_mut();
    let mut data = null_mut();

    let rc = unsafe {
        rrd_sys::rrd_fetch_r(
            filename.as_ptr(),
            cf.as_ptr(),
            &mut start,
            &mut end,
            &mut resolution,
            &mut ds_count,
            &mut ds_names,
            &mut data,
        )
    };
    return_code_to_result(rc)?;

    assert!(!ds_names.is_null());
    assert!(!data.is_null());
    assert!(resolution > 0);

    // Move forward one step -- first timestamp's data is included in the time that ends one step ahead
    let start = Timestamp::from_time_t(
        start
            .checked_add(i64::try_from(resolution).expect("Resolution i64 overflow"))
            .expect("Start overflow"),
    );
    let end = Timestamp::from_time_t(end);

    let ds_count_usize = ds_count.try_into().expect("Count overflow");

    let names = unsafe {
        let names: Vec<_> = slice::from_raw_parts(ds_names, ds_count_usize)
            .iter()
            .map(|p| {
                let s = CStr::from_ptr(*p).to_string_lossy().into_owned();
                rrd_sys::rrd_freemem(*p as *mut rrd_void);
                s
            })
            .collect();
        rrd_sys::rrd_freemem(ds_names as *mut rrd_void);
        names
    };

    let rows = (usize::try_from(
        end.as_time_t()
            .checked_sub(start.as_time_t())
            .expect("Negative time range"),
    )
    .expect("Time range overflow")
        / usize::try_from(resolution).expect("Resolution usize overflow"))
    .checked_add(1)
    .expect("Num rows overflow");
    let data = Array {
        ptr: data,
        len: rows.checked_mul(ds_count_usize).expect("Data len overflow"),
    };

    // we need u64, but windows c_ulong is u32
    #[allow(clippy::useless_conversion)]
    Ok(Data::new(
        start,
        end,
        Duration::from_secs(resolution.into()),
        names,
        data,
    ))
}

/// Contiguous data for the output of [`fetch`].
///
/// This is not intended to be used directly, but rather is the underlying storage accessed via
/// [`Data`].
pub struct Array {
    ptr: *const rrd_double,
    len: usize,
}

impl Drop for Array {
    fn drop(&mut self) {
        unsafe {
            rrd_sys::rrd_freemem(self.ptr as *mut rrd_void);
        }
    }
}

unsafe impl Send for Array {}

impl Deref for Array {
    type Target = [rrd_double];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl fmt::Debug for Array {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.deref().iter()).finish()
    }
}
