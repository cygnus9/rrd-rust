use std::{
    ffi::CString,
    path::Path,
    ptr::null,
    time::{Duration, SystemTime},
};

use rrd_sys::{rrd_int, rrd_ulong};

use crate::{
    error::{
        return_code_to_result,
        RrdResult
    },
    util::{self, path_to_str, ArrayOfStrings, NullTerminatedArrayOfStrings}
};

pub fn create(
    filename: &Path,
    pdp_step: Duration,
    last_up: SystemTime,
    no_overwrite: bool,
    sources: &[&Path],
    template: Option<&Path>,
    args: &[&str],
) -> RrdResult<()> {
    let filename = CString::new(path_to_str(filename)?)?;
    let sources = sources
        .iter()
        .map(|p| path_to_str(p))
        .collect::<Result<Vec<_>, _>>()?;
    let sources = NullTerminatedArrayOfStrings::new(sources)?;
    let template = match template {
        None => None,
        Some(p) => Some(CString::new(path_to_str(p)?)?),
    };
    let args = ArrayOfStrings::new(args)?;

    let rc = unsafe {
        rrd_sys::rrd_create_r2(
            filename.as_ptr(),
            pdp_step.as_secs() as rrd_ulong,
            util::to_unix_time(last_up).unwrap(),
            if no_overwrite { 1 } else { 0 },
            sources.as_ptr(),
            template.map_or(null(), |s| s.as_ptr()),
            args.len() as rrd_int,
            args.as_ptr(),
        )
    };
    return_code_to_result(rc)
}
