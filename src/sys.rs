#[allow(non_camel_case_types)]
pub type c_char = std::os::raw::c_char;
#[allow(non_camel_case_types)]
pub type c_int = std::os::raw::c_int;
#[allow(non_camel_case_types)]
pub type c_time_t = libc::time_t;
#[allow(non_camel_case_types)]
pub type c_ulong = std::os::raw::c_ulong;

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

    pub fn rrd_get_error() -> *const c_char;
}
