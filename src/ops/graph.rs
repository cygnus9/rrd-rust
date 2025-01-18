use crate::error::{
    return_code_to_result,
    RrdResult
};
use rrd_sys::FILE;
use std::ffi::{c_char, c_int, CString};

pub fn graph(filename: &str, args: Vec<&str>) -> RrdResult<()> {
    let c_filename = CString::new(filename)?;
    let args: Vec<_> = args
        .iter()
        .map(|s| CString::new(*s))
        .collect::<Result<Vec<_>, _>>()?;

    let rrd_graph_str = CString::new("rrd_graph")?;

    let mut argv = vec![rrd_graph_str.as_ptr(), c_filename.as_ptr()];
    argv.extend(args.iter().map(|s| s.as_ptr()));

    let argc = argv.len() as c_int;

    let mut prdata: *mut *mut c_char = std::ptr::null_mut();
    let mut xsize: c_int = 0;
    let mut ysize: c_int = 0;
    let stream: *mut FILE = std::ptr::null_mut();
    let mut ymin: f64 = 0.0;
    let mut ymax: f64 = 0.0;

    let rc = unsafe {
        rrd_sys::rrd_graph(
            argc,
            // cast to allow different systems that expect *mut/*const pointers
            argv.as_mut_ptr() as _,
            &mut prdata,
            &mut xsize,
            &mut ysize,
            stream,
            &mut ymin,
            &mut ymax,
        )
    };

    // prove that the strings live past the graph call
    drop(rrd_graph_str);
    drop(args);
    drop(c_filename);

    return_code_to_result(rc)
}
