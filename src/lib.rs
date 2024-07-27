use bitflags::bitflags;
use data::Data;
use std::{
    ffi::{CStr, CString},
    ops::Deref,
    path::Path,
    ptr::{null, null_mut},
    slice,
    time::{Duration, SystemTime},
};

use crate::{
    error::{RrdError, RrdResult},
    util::{path_to_str, ArrayOfStrings, NullTerminatedArrayOfStrings},
};

pub mod data;
pub mod error;
pub mod util;

pub fn create(
    filename: &Path,
    pdp_step: Duration,
    last_up: SystemTime,
    no_overwrite: bool,
    sources: &[&Path],
    template: Option<&Path>,
    args: &[&str],
) -> RrdResult<()> {
    let filename = CString::new(path_to_str(filename)?)?;
    let sources = sources
        .iter()
        .map(|p| path_to_str(p))
        .collect::<Result<Vec<_>, _>>()?;
    let sources = NullTerminatedArrayOfStrings::new(sources)?;
    let template = match template {
        None => None,
        Some(p) => Some(CString::new(path_to_str(p)?)?),
    };
    let args = ArrayOfStrings::new(args)?;

    let rc = unsafe {
        rrd_sys::rrd_create_r2(
            filename.as_ptr(),
            pdp_step.as_secs() as rrd_sys::c_ulong,
            util::to_unix_time(last_up).unwrap(),
            if no_overwrite { 1 } else { 0 },
            sources.as_ptr(),
            template.map_or(null(), |s| s.as_ptr()),
            args.len() as rrd_sys::c_int,
            args.as_ptr(),
        )
    };
    match rc {
        0 => Ok(()),
        _ => Err(RrdError::LibRrdError(get_error())),
    }
}

pub struct Array {
    ptr: *const rrd_sys::c_double,
    len: usize,
}

impl Drop for Array {
    fn drop(&mut self) {
        unsafe {
            rrd_sys::rrd_freemem(self.ptr as *mut rrd_sys::c_void);
        }
    }
}

impl Deref for Array {
    type Target = [rrd_sys::c_double];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }
}

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
    let mut step = step.as_secs() as rrd_sys::c_ulong;

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
                rrd_sys::rrd_freemem(*p as *mut rrd_sys::c_void);
                s
            })
            .collect();
        rrd_sys::rrd_freemem(ds_names as *mut rrd_sys::c_void);
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

bitflags! {
    pub struct ExtraFlags : rrd_sys::c_int {
        const SKIP_PAST_UPDATES = 0x01;
    }
}

pub fn update(
    filename: &Path,
    template: Option<&Path>,
    extra_flags: ExtraFlags,
    args: &[&str],
) -> RrdResult<()> {
    let filename = CString::new(path_to_str(filename)?)?;
    let template = match template {
        None => None,
        Some(p) => Some(CString::new(path_to_str(p)?)?),
    };
    let args = ArrayOfStrings::new(args)?;
    let rc = unsafe {
        rrd_sys::rrd_updatex_r(
            filename.as_ptr(),
            template.map_or(null(), |s| s.as_ptr()),
            extra_flags.bits(),
            args.len() as rrd_sys::c_int,
            args.as_ptr(),
        )
    };
    match rc {
        0 => Ok(()),
        _ => Err(RrdError::LibRrdError(get_error())),
    }
}

fn get_error() -> String {
    unsafe {
        let p = rrd_sys::rrd_get_error();
        let s = CStr::from_ptr(p);
        s.to_str().unwrap().to_owned()
    }
}
