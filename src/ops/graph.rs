use crate::{
    error::{RrdError, RrdResult},
    get_error,
};
use rrd_sys::FILE;
use std::ffi::{c_char, c_int, CString};

pub fn graph(filename: &str, args: Vec<&str>) -> RrdResult<()> {
    let c_filename = CString::new(filename).unwrap();
    let arg_ptrs: Vec<_> = args
        .iter()
        .map(|s| CString::new(*s).unwrap().into_raw())
        .collect();

    let rrd_graph_str = CString::new("rrd_graph").unwrap();
    let rrd_graph_str_ptr = rrd_graph_str.into_raw();

    let mut argv = vec![rrd_graph_str_ptr, c_filename.into_raw()];
    argv.extend(arg_ptrs.iter().map(|&s| s as *mut c_char));

    let argc = argv.len() as c_int;

    let mut prdata: *mut *mut c_char = std::ptr::null_mut();
    let mut xsize: c_int = 0;
    let mut ysize: c_int = 0;
    let stream: *mut FILE = std::ptr::null_mut();
    let mut ymin: f64 = 0.0;
    let mut ymax: f64 = 0.0;

    let res = unsafe {
        rrd_sys::rrd_graph(
            argc,
            argv.as_mut_ptr(),
            &mut prdata,
            &mut xsize,
            &mut ysize,
            stream,
            &mut ymin,
            &mut ymax,
        )
    };
    match res {
        0 => Ok(()),
        _ => Err(RrdError::LibRrdError(get_error())),
    }
}
