use std::collections::HashMap;
use std::ffi::{CStr, CString};

pub fn info(filename: &str) -> HashMap<String, String> {
    let filename = CString::new(filename).expect("CString::new failed");

    let result_ptr = unsafe { rrd_sys::rrd_info_r(filename.as_ptr()) };
    let mut dict = HashMap::new();
    let mut current = result_ptr;
    while !current.is_null() {
        let key_cstr = unsafe { CStr::from_ptr((*current).key) };
        let key = key_cstr.to_string_lossy().into_owned();

        let value = match unsafe { &(*current).type_ } {
            &rrd_sys::rrd_info_type_RD_I_VAL => {
                let value = unsafe { (*current).value.u_val };
                format!("{:.2}", value) // Handle value as a double with 2 decimal places
            }
            &rrd_sys::rrd_info_type_RD_I_CNT => {
                let value = unsafe { (*current).value.u_cnt };
                format!("{}", value) // Handle value as an unsigned int
            }
            &rrd_sys::rrd_info_type_RD_I_STR => {
                let str_cstr = unsafe { CStr::from_ptr((*current).value.u_str) };
                str_cstr.to_string_lossy().into_owned() // Handle value as a string
            }
            &rrd_sys::rrd_info_type_RD_I_INT => {
                let value = unsafe { (*current).value.u_int };
                format!("{}", value) // Handle value as an int
            }
            &rrd_sys::rrd_info_type_RD_I_BLO => {
                // Skip handling blobs for simplicity
                continue;
            }
            _ => {
                continue;
            }
        };

        dict.insert(key, value);

        current = unsafe { (*current).next };
    }

    dict
}
