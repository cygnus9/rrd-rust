#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub use core::ffi::c_char as rrd_char;
pub use core::ffi::c_double as rrd_double;
pub use core::ffi::c_int as rrd_int;
pub use core::ffi::c_ulong as rrd_ulong;
pub use core::ffi::c_void as rrd_void;

#[cfg(rrdsys_use_pregen)]
include!("pregen/bindings.rs");
#[cfg(not(rrdsys_use_pregen))]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
