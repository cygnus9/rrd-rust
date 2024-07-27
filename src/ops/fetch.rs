use std::{
    ffi::{CStr, CString},
    ops::Deref,
    path::Path,
    ptr::null_mut,
    slice,
    time::{Duration, SystemTime},
};

use rrd_sys::{rrd_double, rrd_ulong, rrd_void};

use crate::{
    data::Data,
    error::{RrdError, RrdResult},
    get_error,
    util::{self, path_to_str},
};

pub fn fetch(
    filename: &Path,
    cf: &str,
    start: SystemTime,
    end: SystemTime,
    step: Duration,
) -> RrdResult<Data<Array>> {
    // in
    let filename = CString::new(path_to_str(filename)?)?;
    let cf = CString::new(cf)?;

    // in/out
    let mut start = util::to_unix_time(start).unwrap();
    let mut end = util::to_unix_time(end).unwrap();
    let mut step = step.as_secs() as rrd_ulong;

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
            &mut step,
            &mut ds_count,
            &mut ds_names,
            &mut data,
        )
    };
    if rc != 0 {
        return Err(RrdError::LibRrdError(get_error()));
    }

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

    let rows = (end - start) as usize / step as usize + 1;
    let data = Array {
        ptr: data,
        len: rows * ds_count as usize,
    };

    Ok(Data::new(
        util::from_unix_time(start),
        util::from_unix_time(end),
        Duration::from_secs(step as u64),
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
