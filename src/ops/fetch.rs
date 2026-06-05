//! Fetch data from an RRD.

use crate::{
    data::Data,
    error::{return_code_to_result, RrdError, RrdResult},
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
///
/// # Errors
/// Returns an error if the RRD file cannot be read or if the fetch operation fails.
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
    let mut start = start.try_as_time_t()?;
    let mut end = end.try_as_time_t()?;
    #[allow(clippy::useless_conversion)]
    let mut resolution = resolution
        .as_secs()
        .try_into()
        .map_err(|_| RrdError::InvalidArgument("resolution is too large for librrd".to_string()))?;

    // out
    let mut ds_count = 0;
    let mut ds_names = null_mut();
    let mut data = null_mut();

    let rc = unsafe {
        rrd_sys::rrd_fetch_r(
            filename.as_ptr(),
            cf.as_ptr(),
            &raw mut start,
            &raw mut end,
            &raw mut resolution,
            &raw mut ds_count,
            &raw mut ds_names,
            &raw mut data,
        )
    };
    return_code_to_result(rc)?;

    if ds_names.is_null() {
        return Err(RrdError::Internal(
            "librrd fetch returned null data source names".to_string(),
        ));
    }
    if data.is_null() {
        return Err(RrdError::Internal(
            "librrd fetch returned null data".to_string(),
        ));
    }
    if resolution == 0 {
        return Err(RrdError::Internal(
            "librrd fetch returned zero resolution".to_string(),
        ));
    }

    // Move forward one step -- first timestamp's data is included in the time that ends one step ahead
    let resolution_i64 = i64::try_from(resolution).map_err(|_| {
        RrdError::Internal(format!(
            "librrd fetch returned resolution {resolution} that overflows i64"
        ))
    })?;
    let start_time_t = start
        .checked_add(resolution_i64)
        .ok_or_else(|| RrdError::Internal("Fetch start timestamp overflow".to_string()))?;
    let end_time_t = end;
    let start = Timestamp::try_from_time_t(start_time_t)?;
    let end = Timestamp::try_from_time_t(end_time_t)?;

    let ds_count_usize = ds_count.try_into().map_err(|_| {
        RrdError::Internal(format!("librrd fetch returned invalid DS count {ds_count}"))
    })?;

    let names = unsafe {
        let names: Vec<_> = slice::from_raw_parts(ds_names, ds_count_usize)
            .iter()
            .map(|p| {
                let s = CStr::from_ptr(*p).to_string_lossy().into_owned();
                rrd_sys::rrd_freemem((*p).cast::<rrd_void>());
                s
            })
            .collect();
        rrd_sys::rrd_freemem(ds_names.cast::<rrd_void>());
        names
    };

    let time_range = end_time_t
        .checked_sub(start_time_t)
        .ok_or_else(|| RrdError::Internal("Negative fetch time range".to_string()))?;
    let time_range = usize::try_from(time_range)
        .map_err(|_| RrdError::Internal("Fetch time range overflow".to_string()))?;
    let resolution = usize::try_from(resolution)
        .map_err(|_| RrdError::Internal("Fetch resolution overflow".to_string()))?;
    let rows = (time_range / resolution)
        .checked_add(1)
        .ok_or_else(|| RrdError::Internal("Fetch row count overflow".to_string()))?;
    let data = Array {
        ptr: data,
        len: rows
            .checked_mul(ds_count_usize)
            .ok_or_else(|| RrdError::Internal("Fetch data length overflow".to_string()))?,
    };

    let resolution = u64::try_from(resolution)
        .map_err(|_| RrdError::Internal("Fetch resolution overflow".to_string()))?;
    Ok(Data::new(
        start,
        end,
        Duration::from_secs(resolution),
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
