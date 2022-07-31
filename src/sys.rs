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
}
