#[allow(non_camel_case_types)]
pub type c_char = std::os::raw::c_char;
#[allow(non_camel_case_types)]
pub type c_double = std::os::raw::c_double;
#[allow(non_camel_case_types)]
pub type c_int = std::os::raw::c_int;
#[allow(non_camel_case_types)]
pub type c_time_t = libc::time_t;
#[allow(non_camel_case_types)]
pub type c_ulong = std::os::raw::c_ulong;
#[allow(non_camel_case_types)]
pub type c_void = std::os::raw::c_void;
pub type rrd_value_t = c_double;

use std::os::raw::c_uchar;
use libc::FILE;

#[repr(C)]
pub struct rrd_blob_t {
    pub size: c_ulong,
    pub ptr: *mut c_uchar,
}

#[repr(C)]
pub union rrd_infoval_t {
    pub u_cnt: c_ulong,
    pub u_val: rrd_value_t,
    pub u_str: *mut c_char,
    pub u_int: c_int,
    pub u_blo: std::mem::ManuallyDrop<rrd_blob_t>,
}


#[repr(C)]
pub enum rrd_info_type_t {
    RD_I_VAL = 0,
    RD_I_CNT,
    RD_I_STR,
    RD_I_INT,
    RD_I_BLO,
}

#[repr(C)]
pub struct rrd_info_t {
    pub key: *mut c_char,
    pub type_: rrd_info_type_t,
    pub value: rrd_infoval_t,
    pub next: *mut rrd_info_t,
}


extern "C" {
    pub fn rrd_create_r2(
        filename: *const c_char,
        pdp_step: c_ulong,
        last_up: c_time_t,
        no_overwrite: c_int,
        sources: *const *const c_char,
        template: *const c_char,
        argc: c_int,
        argv: *const *const c_char,
    ) -> c_int;

    pub fn rrd_fetch_r(
        filename: *const c_char,
        cf: *const c_char,
        start: *mut c_time_t,
        end: *mut c_time_t,
        step: *mut c_ulong,
        ds_count: *mut c_ulong,
        ds_names: *mut *const *const c_char,
        data: *mut *const c_double,
    ) -> c_int;

    pub fn rrd_freemem(mem: *mut c_void);

    pub fn rrd_get_error() -> *const c_char;

    pub fn rrd_updatex_r(
        filename: *const c_char,
        template: *const c_char,
        extra_flags: c_int,
        argc: c_int,
        argv: *const *const c_char,
    ) -> c_int;

    pub fn rrd_graph(
        argc: c_int,
        argv: *const *const c_char,
        prdata: *mut *mut *mut c_char,
        xsize: *mut c_int,
        ysize: *mut c_int,
        stream: *mut FILE,
        ymin: *mut f64,
        ymax: *mut f64,
    ) -> c_int;

    pub fn rrd_info_r(filename: *const c_char) -> *mut rrd_info_t;
}
