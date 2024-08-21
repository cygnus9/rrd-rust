pub mod data;
pub mod error;
pub mod util;

pub(crate) mod ops;
use std::ffi::CStr;

pub use ops::create::create;
pub use ops::fetch::fetch;
pub use ops::update::{update, ExtraFlags};

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
        let p = rrd_sys::rrd_get_error();
        let s = CStr::from_ptr(p);
        s.to_str().unwrap().to_owned()
    }
}
