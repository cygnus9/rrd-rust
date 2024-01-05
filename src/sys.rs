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

#[repr(C)]
pub struct rrd_blob_t {
    size: c_ulong,   // size of the blob
    ptr: *mut u8,    // pointer
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
pub union rrd_infoval_t {
    u_cnt: c_ulong,
    u_val: f64,        // assuming rrd_value_t is equivalent to double in C
    u_str: *mut c_char,
    u_int: c_int,
    u_blo: rrd_blob_t,
}

#[repr(C)]
pub struct rrd_info_t {
    key: *mut c_char,
    type_: rrd_info_type_t,
    value: rrd_infoval_t,
    next: *mut rrd_info_t,
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
        prdata: *mut *mut c_char,
        xsize: *mut c_ulong,
        ysize: *mut c_ulong,
        info: *mut *mut rrd_info_t,
    ) -> c_int;
}
