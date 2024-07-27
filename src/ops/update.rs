use std::{ffi::CString, path::Path, ptr::null};

use bitflags::bitflags;
use rrd_sys::rrd_int;

use crate::{
    error::{RrdError, RrdResult},
    get_error,
    util::{path_to_str, ArrayOfStrings},
};

bitflags! {
    pub struct ExtraFlags : rrd_int {
        const SKIP_PAST_UPDATES = 0x01;
    }
}

pub fn update(
    filename: &Path,
    template: Option<&Path>,
    extra_flags: ExtraFlags,
    args: &[&str],
) -> RrdResult<()> {
    let filename = CString::new(path_to_str(filename)?)?;
    let template = match template {
        None => None,
        Some(p) => Some(CString::new(path_to_str(p)?)?),
    };
    let args = ArrayOfStrings::new(args)?;
    let rc = unsafe {
        rrd_sys::rrd_updatex_r(
            filename.as_ptr(),
            template.map_or(null(), |s| s.as_ptr()),
            extra_flags.bits(),
            args.len() as rrd_int,
            args.as_ptr(),
        )
    };
    match rc {
        0 => Ok(()),
        _ => Err(RrdError::LibRrdError(get_error())),
    }
}
