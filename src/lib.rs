use bitflags::bitflags;
use data::Data;
use libc::FILE;
use std::{
    ffi::{CStr, CString},
    ops::Deref,
    path::Path,
    ptr::null,
    slice,
    time::{Duration, SystemTime},
};
use std::collections::HashMap;

use crate::{
    error::{RrdError, RrdResult},
    util::{path_to_str, ArrayOfStrings, NullTerminatedArrayOfStrings},
};
use crate::sys::rrd_info_type_t;

pub mod data;
pub mod error;
mod sys;
pub mod util;

#[allow(non_camel_case_types)]
pub type c_ulong = std::os::raw::c_ulong;
#[allow(non_camel_case_types)]
pub type c_char = std::os::raw::c_char;
#[allow(non_camel_case_types)]
pub type c_int = std::os::raw::c_int;

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
        sys::rrd_create_r2(
            filename.as_ptr(),
            pdp_step.as_secs() as sys::c_ulong,
            util::to_unix_time(last_up).unwrap(),
            if no_overwrite { 1 } else { 0 },
            sources.as_ptr(),
            template.map_or(null(), |s| s.as_ptr()),
            args.len() as sys::c_int,
            args.as_ptr(),
        )
    };
    match rc {
        0 => Ok(()),
        _ => Err(RrdError::LibRrdError(get_error())),
    }
}

pub struct Array {
    ptr: *const sys::c_double,
    len: usize,
}

impl Drop for Array {
    fn drop(&mut self) {
        unsafe {
            sys::rrd_freemem(self.ptr as *mut sys::c_void);
        }
    }
}

impl Deref for Array {
    type Target = [sys::c_double];

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
    let mut step = step.as_secs() as sys::c_ulong;

    // out
    let mut ds_count = 0;
    let mut ds_names = null();
    let mut data = null();

    let rc = unsafe {
        sys::rrd_fetch_r(
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
                sys::rrd_freemem(*p as *mut sys::c_void);
                s
            })
            .collect();
        sys::rrd_freemem(ds_names as *mut sys::c_void);
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
    pub struct ExtraFlags : sys::c_int {
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
        sys::rrd_updatex_r(
            filename.as_ptr(),
            template.map_or(null(), |s| s.as_ptr()),
            extra_flags.bits(),
            args.len() as sys::c_int,
            args.as_ptr(),
        )
    };
    match rc {
        0 => Ok(()),
        _ => Err(RrdError::LibRrdError(get_error())),
    }
}

pub fn graph(filename: &str, args: Vec<&str>) -> RrdResult<()> {
    let c_filename = CString::new(filename).unwrap();
    let arg_ptrs: Vec<_> = args.iter()
        .map(|s| CString::new(*s).unwrap())
        .collect();

    let rrd_graph_str = CString::new("rrd_graph").unwrap();
    let rrd_graph_str_ptr = rrd_graph_str.as_ptr();

    let mut argv = vec![rrd_graph_str_ptr, c_filename.as_ptr()];
    argv.extend(arg_ptrs.iter().map(|s| s.as_ptr()));

    let argc = argv.len() as c_int;

    let mut prdata: *mut *mut c_char = std::ptr::null_mut();
    let mut xsize: c_int = 0;
    let mut ysize: c_int = 0;
    let stream: *mut FILE = std::ptr::null_mut();
    let mut ymin: f64 = 0.0;
    let mut ymax: f64 = 0.0;

    let res = unsafe {
        sys::rrd_graph(argc, argv.as_ptr(), &mut prdata, &mut xsize, &mut ysize, stream, &mut ymin, &mut ymax)
    };
    match res {
        0 => Ok(()),
        _ => Err(RrdError::LibRrdError(get_error())),
    }
}

pub fn info(filename: &str) -> HashMap<String,String> {
    let filename = CString::new(filename).expect("CString::new failed");

    let result_ptr = unsafe {
        sys::rrd_info_r(filename.as_ptr())
    };
    let mut dict = HashMap::new();
    let mut current = result_ptr;
    while !current.is_null() {
        let key_cstr = unsafe { CStr::from_ptr((*current).key) };
        let key = key_cstr.to_string_lossy().into_owned();

        let value = match unsafe { &(*current).type_ } {
            &rrd_info_type_t::RD_I_VAL => {
                let value = unsafe { (*current).value.u_val };
                format!("{:.2}", value) // Handle value as a double with 2 decimal places
            }
            &rrd_info_type_t::RD_I_CNT => {
                let value = unsafe { (*current).value.u_cnt };
                format!("{}", value) // Handle value as an unsigned int
            }
            &rrd_info_type_t::RD_I_STR => {
                let str_cstr = unsafe { CStr::from_ptr((*current).value.u_str) };
                str_cstr.to_string_lossy().into_owned() // Handle value as a string
            }
            &rrd_info_type_t::RD_I_INT => {
                let value = unsafe { (*current).value.u_int };
                format!("{}", value) // Handle value as an int
            }
            &rrd_info_type_t::RD_I_BLO => {
                // Skip handling blobs for simplicity
                continue;
            }
        };

        dict.insert(key, value);

        current = unsafe { (*current).next };
    }

    dict

}

fn get_error() -> String {
    unsafe {
        let p = sys::rrd_get_error();
        let s = CStr::from_ptr(p);
        s.to_str().unwrap().to_owned()
    }
}
