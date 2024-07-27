#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub use core::ffi::c_char;
pub use core::ffi::c_double;
pub use core::ffi::c_int;
pub use core::ffi::c_ulong;
pub use core::ffi::c_void;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
