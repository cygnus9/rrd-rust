use std::{
    ffi::{CStr, CString},
    ops::Deref,
    path::Path,
    ptr::null_mut,
    slice,
    time::Duration,
};

use rrd_sys::{rrd_double, rrd_ulong, rrd_void};

use crate::{
    data::Data,
    error::{return_code_to_result, RrdResult},
    util::path_to_str,
    Timestamp, TimestampExt,
};

pub fn fetch(
    filename: &Path,
    cf: &str,
    start: Timestamp,
    end: Timestamp,
    step: Duration,
) -> RrdResult<Data<Array>> {
    // in
    let filename = CString::new(path_to_str(filename)?)?;
    let cf = CString::new(cf)?;

    // in/out
    let mut start_tt = start.as_time_t();
    let mut end_tt = end.as_time_t();
    let mut step_tt = step.as_secs() as rrd_ulong;

    // out
    let mut ds_count = 0;
    let mut ds_names = null_mut();
    let mut data = null_mut();

    let rc = unsafe {
        rrd_sys::rrd_fetch_r(
            filename.as_ptr(),
            cf.as_ptr(),
            &mut start_tt,
            &mut end_tt,
            &mut step_tt,
            &mut ds_count,
            &mut ds_names,
            &mut data,
        )
    };
    return_code_to_result(rc)?;

    let names = unsafe {
        let names: Vec<_> = slice::from_raw_parts(ds_names, ds_count as usize)
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

    let rows = (end_tt - start_tt) as usize / step_tt as usize + 1;
    let data = Array {
        ptr: data,
        len: rows * ds_count as usize,
    };

    // we need u64, but windows c_ulong is u32
    #[allow(clippy::useless_conversion)]
    Ok(Data::new(
        start,
        end,
        Duration::from_secs(step_tt.into()),
        names,
        data,
    ))
}

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

impl Deref for Array {
    type Target = [rrd_double];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }
}
