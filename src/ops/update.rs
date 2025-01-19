use crate::error::RrdError;
use crate::{
    error::{return_code_to_result, RrdResult},
    util::{path_to_str, ArrayOfStrings},
    Timestamp,
};
use bitflags::bitflags;
use log::debug;
use rrd_sys::rrd_int;
use std::{ffi::CString, fmt::Write, path::Path, ptr::null};

bitflags! {
    pub struct ExtraFlags : rrd_int {
        const SKIP_PAST_UPDATES = 0x01;
    }
}

/// Update all data sources in the RRD.
///
/// Each timestamped batch of data must have a datum for each DS, except for `COMPUTE` data sources.
///
/// Each batch of data must have the same number of data points.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdupdate.en.html>.
pub fn update_all<'a, D, I>(filename: &Path, extra_flags: ExtraFlags, data: I) -> RrdResult<()>
where
    D: AsRef<[Datum]> + 'a,
    I: IntoIterator<Item = &'a (BatchTime, D)>,
{
    let filename = CString::new(path_to_str(filename)?)?;
    let args = build_datum_args(data, None)?;

    debug!(
        "Update: file={} extra_flags=0x{extra_flags:02x} args={args:?}",
        filename.to_string_lossy(),
    );

    let rc = unsafe {
        rrd_sys::rrd_updatex_r(
            filename.as_ptr(),
            null(),
            extra_flags.bits(),
            args.len() as rrd_int,
            args.as_ptr(),
        )
    };
    return_code_to_result(rc)
}

/// Update only the DS names specified in `ds_names`.
///
/// No `COMPUTE` DS names should be included, as those do not have values directly provided. DS
/// names not specified (other than `COMPUTE` DSs) will have `unknown` values applied for the
/// given timestamps.
///
/// `data` is a sequence of timestamps with one datum per DS at that timestamp.
///
///  Each batch of data must have the same number of data points.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdupdate.en.html>.
pub fn update<'a, D, I>(
    filename: &Path,
    ds_names: &[&str],
    extra_flags: ExtraFlags,
    data: I,
) -> RrdResult<()>
where
    D: AsRef<[Datum]> + 'a,
    I: IntoIterator<Item = &'a (BatchTime, D)>,
{
    let filename = CString::new(path_to_str(filename)?)?;

    let template = CString::new(ds_names.iter().fold(String::new(), |mut accum, t| {
        if !accum.is_empty() {
            accum.push(':');
        }
        accum.push_str(t);
        accum
    }))?;

    let args = build_datum_args(data, Some(ds_names.len()))?;

    debug!(
        "Update: file={} template={} extra_flags=0x{extra_flags:02x} args={args:?}",
        filename.to_string_lossy(),
        template.to_string_lossy()
    );

    let rc = unsafe {
        rrd_sys::rrd_updatex_r(
            filename.as_ptr(),
            template.as_ptr(),
            extra_flags.bits(),
            args.len() as rrd_int,
            args.as_ptr(),
        )
    };
    return_code_to_result(rc)
}

/// Ensure that all batches match `expected_len`, if set, otherwise ensure they are all the same
/// len.
fn build_datum_args<'a, D, I>(
    batches: I,
    mut expected_len: Option<usize>,
) -> RrdResult<ArrayOfStrings>
where
    D: AsRef<[Datum]> + 'a,
    I: IntoIterator<Item = &'a (BatchTime, D)>,
{
    let args = batches
        .into_iter()
        .map(|(ts, data)| {
            let slice = data.as_ref();
            let expected = expected_len.get_or_insert(slice.len());
            if slice.len() != *expected {
                return Err(RrdError::InvalidArgument(
                    "Batch sizes don't match".to_string(),
                ));
            }

            // approximate minimum size -- at least we can cut out _some_ allocations
            let mut timestamp_arg = String::with_capacity(slice.len() * 2);

            match ts {
                BatchTime::Now => {
                    timestamp_arg.push('N');
                }
                BatchTime::Specified(ts) => {
                    write!(timestamp_arg, "{}", ts.timestamp())
                        .expect("Writing to a String can't fail");
                }
            }

            for datum in slice {
                timestamp_arg.push(':');
                match datum {
                    Datum::Unspecified => {
                        timestamp_arg.push('U');
                    }
                    Datum::Int(i) => {
                        write!(timestamp_arg, "{}", i).expect("Writing to a String can't fail");
                    }
                    Datum::Float(f) => {
                        write!(timestamp_arg, "{}", f).expect("Writing to a String can't fail");
                    }
                }
            }

            Ok(timestamp_arg)
        })
        .collect::<Result<Vec<_>, RrdError>>()?;

    ArrayOfStrings::new(args)
}

/// The value for an individual DS at a particular timestamp
pub enum Datum {
    Unspecified,
    Int(u64),
    Float(f64),
}

impl From<u64> for Datum {
    fn from(value: u64) -> Self {
        Self::Int(value)
    }
}

impl From<f64> for Datum {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

/// Timestamp to use for a batch of values
pub enum BatchTime {
    /// Let RRDTool determine the time from the system clock.
    Now,
    /// Use a specific time
    Specified(Timestamp),
}

impl From<Timestamp> for BatchTime {
    fn from(value: Timestamp) -> Self {
        Self::Specified(value)
    }
}
