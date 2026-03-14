//! Update (i.e. add data to) an RRD.

use crate::error::RrdError;
use crate::TimestampExt;
use crate::{
    error::{return_code_to_result, RrdResult},
    util::{path_to_str, ArrayOfStrings},
    Timestamp,
};
use bitflags::bitflags;
use itertools::Itertools;
use log::debug;
use rrd_sys::rrd_int;
use std::{borrow, ffi::CString, fmt::Write, path::Path, ptr::null};

bitflags! {
    /// Flags to alter update behavior.
    ///
    /// # Examples
    ///
    /// No flags:
    /// ```
    /// use rrd::ops::update::ExtraFlags;
    /// let no_flags = ExtraFlags::empty();
    /// ```
    #[derive(Clone, Copy)]
    pub struct ExtraFlags : rrd_int {
        /// Silently skip updates older than the last update already present rather than returning
        /// an error.
        const SKIP_PAST_UPDATES = 0x01;
    }
}

/// Options to alter update behavior.
///
/// This is an alternative to using `ExtraFlags`.
#[derive(Debug, Default, Clone, Copy)]
pub struct Options {
    /// Silently skip updates older than the last update already present rather than returning an error.
    pub skip_past_updates: bool,
    /// Locking behavior when updating the RRD.
    #[cfg(feature = "locking_mode")]
    pub locking_mode: LockingMode,
}

impl Options {
    /// Get the bitflags representation of these options.
    pub fn bits(&self) -> rrd_int {
        let mut bits = 0;
        if self.skip_past_updates {
            bits |= 1;
        }
        #[cfg(feature = "locking_mode")]
        {
            bits |= match self.locking_mode {
                LockingMode::DEFAULT => 0,
                LockingMode::NONE => 1 << 7,
                LockingMode::BLOCK => 2 << 7,
                LockingMode::TRY => 3 << 7,
            };
        }
        bits
    }
}

impl From<ExtraFlags> for Options {
    fn from(flags: ExtraFlags) -> Self {
        Self {
            skip_past_updates: flags.contains(ExtraFlags::SKIP_PAST_UPDATES),
            #[cfg(feature = "locking_mode")]
            locking_mode: LockingMode::default(),
        }
    }
}

/// Locking behavior when updating the RRD.
#[cfg(feature = "locking_mode")]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum LockingMode {
    /// Read $RRD_LOCKING environment or fall back to TRY.
    #[default]
    DEFAULT,
    /// No locking; caller is responsible to ensure that the file is not used elsewhere.
    NONE,
    /// Wait until lock is available.
    BLOCK,
    /// Try to lock but fail when file is used elsewhere (default).
    TRY,
}

/// Update all data sources in the RRD.
///
/// Each timestamped batch of data must have a datum for each DS, except for `COMPUTE` data sources.
///
/// Each batch of data must have the same number of data points.
///
/// This corresponds to `rrdtool update` without the `--template` parameter.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdupdate.en.html>.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use rrd::error::RrdResult;
/// use rrd::ops::update::{update_all, BatchTime, ExtraFlags};
///
/// fn add_some_data(f: &Path) -> RrdResult<()> {
///     update_all(
///         f,
///         ExtraFlags::empty(),
///         // 1 data point per DS at each timestamp
///         &[(BatchTime::Now, &[1_u64.into(), 2_f64.into()])])
/// }
/// ```
pub fn update_all<'a, D, B, I, O>(filename: &Path, update_options: O, data: I) -> RrdResult<()>
where
    D: AsRef<[Datum]> + 'a,
    B: borrow::Borrow<(BatchTime, D)>,
    I: IntoIterator<Item = B>,
    O: Into<Options>,
{
    let filename = CString::new(path_to_str(filename)?)?;
    let args = build_datum_args(data, None)?;
    let extra_flags = update_options.into().bits();

    debug!("Update: file={filename:?} extra_flags=0x{extra_flags:02x} args={args:?}",);

    let rc = unsafe {
        rrd_sys::rrd_updatex_r(
            filename.as_ptr(),
            null(),
            extra_flags,
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
/// This corresponds to `rrdtool update` with the `--template` parameter.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdupdate.en.html>.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use rrd::error::RrdResult;
/// use rrd::ops::update::{update, BatchTime, ExtraFlags};
///
/// fn add_some_data(f: &Path) -> RrdResult<()> {
///     update(
///         f,
///         // Other DSs will have "unknown" data at the provided timestamps
///         &["ds2"],
///         ExtraFlags::empty(),
///         // 1 data point per listed DS above at each timestamp
///         &[(BatchTime::Now, &[2_f64.into()])])
/// }
/// ```
pub fn update<'a, D, B, I>(
    filename: &Path,
    ds_names: &[&str],
    extra_flags: ExtraFlags,
    data: I,
) -> RrdResult<()>
where
    D: AsRef<[Datum]> + 'a,
    B: borrow::Borrow<(BatchTime, D)>,
    I: IntoIterator<Item = B>,
{
    let filename = CString::new(path_to_str(filename)?)?;
    let template = CString::new(ds_names.iter().join(":"))?;
    let args = build_datum_args(data, Some(ds_names.len()))?;

    debug!(
        "Update: file={filename:?} template={template:?} extra_flags=0x{extra_flags:02x} args={args:?}",
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

/// The value to set for an individual DS at a particular timestamp.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
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

/// Timestamp to use for a batch of [`Datum`] values in an update call.
pub enum BatchTime {
    /// Let `librrd` determine the time from the system clock.
    Now,
    /// Use a specific time
    Timestamp(Timestamp),
}

impl From<Timestamp> for BatchTime {
    fn from(value: Timestamp) -> Self {
        Self::Timestamp(value)
    }
}

/// Ensure that all batches match `expected_len`, if set, otherwise ensure they are all the same
/// len.
fn build_datum_args<'a, D, B, I>(
    batches: I,
    mut expected_len: Option<usize>,
) -> RrdResult<ArrayOfStrings>
where
    D: AsRef<[Datum]> + 'a,
    B: borrow::Borrow<(BatchTime, D)>,
    I: IntoIterator<Item = B>,
{
    batches
        .into_iter()
        .map(|batch| {
            let (ts, data) = batch.borrow();
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
                BatchTime::Timestamp(ts) => {
                    write!(timestamp_arg, "{}", ts.as_time_t())
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

            CString::new(timestamp_arg).map_err(|e| e.into())
        })
        .collect::<Result<ArrayOfStrings, _>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::create;
    use crate::ConsolidationFn;
    use std::time;

    #[test]
    fn can_call_update_with_tuple_refs() -> anyhow::Result<()> {
        let tempdir = tempfile::tempdir()?;
        let rrd_path = tempdir.path().join("data.rrd");

        create(&rrd_path)?;

        call_update_with_tuple_refs(
            &rrd_path,
            &[(Timestamp::from_time_t(920804460).into(), [100_u64.into()])],
        )?;

        Ok(())
    }

    #[test]
    fn can_call_update_with_tuple_vals() -> anyhow::Result<()> {
        let tempdir = tempfile::tempdir()?;
        let rrd_path = tempdir.path().join("data.rrd");

        create(&rrd_path)?;

        call_update_with_tuple_vals(
            &rrd_path,
            [(Timestamp::from_time_t(920804460).into(), [100_u64.into()])],
        )?;

        Ok(())
    }

    fn create(rrd_path: &Path) -> anyhow::Result<()> {
        create::create(
            rrd_path,
            Timestamp::from_time_t(920804400),
            time::Duration::from_secs(300),
            true,
            None,
            &[],
            &[create::DataSource::counter(
                create::DataSourceName::new("speed"),
                600,
                None,
                None,
            )],
            &[
                create::Archive::new(ConsolidationFn::Avg, 0.5, 1, 24)?,
                create::Archive::new(ConsolidationFn::Avg, 0.5, 6, 10)?,
            ],
        )?;

        Ok(())
    }

    fn call_update_with_tuple_refs<'a, I>(rrd_path: &Path, data: I) -> RrdResult<()>
    where
        I: IntoIterator<Item = &'a (BatchTime, [Datum; 1])>,
    {
        update_all(rrd_path, Options::default(), data)
    }

    fn call_update_with_tuple_vals(
        rrd_path: &Path,
        data: impl IntoIterator<Item = (BatchTime, [Datum; 1])>,
    ) -> RrdResult<()> {
        update_all(rrd_path, Options::default(), data)
    }

    #[test]
    fn convert_extra_flags_to_options() {
        for flags in [ExtraFlags::empty(), ExtraFlags::SKIP_PAST_UPDATES] {
            let options: Options = flags.into();
            assert_eq!(
                options.skip_past_updates,
                flags.contains(ExtraFlags::SKIP_PAST_UPDATES)
            );
            #[cfg(feature = "locking_mode")]
            assert_eq!(options.locking_mode, LockingMode::default());
            assert_eq!(options.bits(), flags.bits());
        }
    }

    #[cfg(feature = "locking_mode")]
    #[test]
    fn options_locking_mode_bits() {
        use LockingMode::*;
        let cases = [(DEFAULT, 0), (NONE, 1 << 7), (BLOCK, 2 << 7), (TRY, 3 << 7)];
        for (mode, expected_bits) in cases {
            let options = Options {
                locking_mode: mode,
                ..Default::default()
            };
            assert_eq!(options.bits(), expected_bits);
        }
    }
}
