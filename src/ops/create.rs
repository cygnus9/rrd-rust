//! Create new RRDs.

use crate::{
    error::{return_code_to_result, InvalidArgument, RrdError, RrdResult},
    util::{path_to_str, ArrayOfStrings, NullTerminatedArrayOfStrings},
    ConsolidationFn, Timestamp, TimestampExt,
};
use log::debug;
use std::convert;
use std::{ffi::CString, path::Path, ptr::null, time::Duration};

/// Create a new RRD.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdcreate.en.html>.
///
/// # Errors
/// Returns an error if the RRD file cannot be created or if any provided paths are invalid.
#[allow(clippy::too_many_arguments)]
pub fn create<'a>(
    filename: &Path,
    start: Timestamp,
    step: Duration,
    no_overwrite: bool,
    template: Option<&Path>,
    sources: &[&Path],
    data_sources: impl IntoIterator<Item = &'a DataSource>,
    round_robin_archives: impl IntoIterator<Item = &'a Archive>,
) -> RrdResult<()> {
    let sources = sources
        .iter()
        .map(|p| path_to_str(p).and_then(|s| CString::new(s).map_err(convert::Into::into)))
        .collect::<Result<NullTerminatedArrayOfStrings, _>>()?;
    let filename = CString::new(path_to_str(filename)?)?;
    let template = match template {
        None => None,
        Some(p) => Some(CString::new(path_to_str(p)?)?),
    };

    let args = data_sources
        .into_iter()
        .map(DataSource::as_arg_string)
        .chain(round_robin_archives.into_iter().map(Archive::as_arg_string))
        .map(CString::new)
        .collect::<Result<ArrayOfStrings, _>>()?;

    let start = start.try_as_time_t()?;

    debug!(
        "Create: file={filename:?} start={} step={} no_overwrite={no_overwrite} template={template:?} sources={sources:?} args={args:?}",
        start,
        step.as_secs()
    );

    #[allow(clippy::useless_conversion)]
    let step = step
        .as_secs()
        .try_into()
        .map_err(|_| RrdError::InvalidArgument("step is too large for librrd".to_string()))?;
    let argc = args.len().try_into().map_err(|_| {
        RrdError::InvalidArgument("too many create arguments for librrd".to_string())
    })?;

    let rc = unsafe {
        rrd_sys::rrd_create_r2(
            filename.as_ptr(),
            step,
            start,
            no_overwrite.into(),
            sources.as_ptr(),
            template.map_or_else(null, |s| s.as_ptr()),
            argc,
            args.as_ptr(),
        )
    };
    return_code_to_result(rc)
}

/// Definition of a data source in an RRD.
///
/// Corresponds to the `DS` arg to `rrdcreate`.
pub struct DataSource {
    arg: String,
}

impl DataSource {
    /// Define a 'GAUGE' data source.
    #[must_use]
    pub fn gauge(
        name: &DataSourceName,
        heartbeat: u32,
        min: Option<f64>,
        max: Option<f64>,
    ) -> Self {
        Self {
            arg: format!(
                "DS:{}:GAUGE:{heartbeat}:{}:{}",
                name.name,
                min.map_or_else(|| "U".to_string(), |m| m.to_string()),
                max.map_or_else(|| "U".to_string(), |m| m.to_string())
            ),
        }
    }

    /// Define a 'COUNTER' data source.
    #[must_use]
    pub fn counter(
        name: &DataSourceName,
        heartbeat: u32,
        min: Option<u64>,
        max: Option<u64>,
    ) -> Self {
        Self {
            arg: format!(
                "DS:{}:COUNTER:{heartbeat}:{}:{}",
                name.name,
                min.map_or_else(|| "U".to_string(), |m| m.to_string()),
                max.map_or_else(|| "U".to_string(), |m| m.to_string())
            ),
        }
    }

    /// Define a 'DCOUNTER' data source.
    #[must_use]
    pub fn dcounter(
        name: &DataSourceName,
        heartbeat: u32,
        min: Option<f64>,
        max: Option<f64>,
    ) -> Self {
        Self {
            arg: format!(
                "DS:{}:DCOUNTER:{heartbeat}:{}:{}",
                name.name,
                min.map_or_else(|| "U".to_string(), |m| m.to_string()),
                max.map_or_else(|| "U".to_string(), |m| m.to_string())
            ),
        }
    }

    /// Define a 'DERIVE' data source.
    #[must_use]
    pub fn derive(
        name: &DataSourceName,
        heartbeat: u32,
        min: Option<u64>,
        max: Option<u64>,
    ) -> Self {
        Self {
            arg: format!(
                "DS:{}:DERIVE:{heartbeat}:{}:{}",
                name.name,
                min.map_or_else(|| "U".to_string(), |m| m.to_string()),
                max.map_or_else(|| "U".to_string(), |m| m.to_string())
            ),
        }
    }

    /// Define a 'DDERIVE' data source.
    #[must_use]
    pub fn dderive(
        name: &DataSourceName,
        heartbeat: u32,
        min: Option<f64>,
        max: Option<f64>,
    ) -> Self {
        Self {
            arg: format!(
                "DS:{}:DDERIVE:{heartbeat}:{}:{}",
                name.name,
                min.map_or_else(|| "U".to_string(), |m| m.to_string()),
                max.map_or_else(|| "U".to_string(), |m| m.to_string())
            ),
        }
    }

    /// Define an 'ABSOLUTE' data source.
    #[must_use]
    pub fn absolute(
        name: &DataSourceName,
        heartbeat: u32,
        min: Option<u64>,
        max: Option<u64>,
    ) -> Self {
        Self {
            arg: format!(
                "DS:{}:ABSOLUTE:{heartbeat}:{}:{}",
                name.name,
                min.map_or_else(|| "U".to_string(), |m| m.to_string()),
                max.map_or_else(|| "U".to_string(), |m| m.to_string())
            ),
        }
    }

    /// Define a 'COMPUTE' data source.
    #[must_use]
    pub fn compute(name: &DataSourceName, rpn: &str) -> Self {
        Self {
            arg: format!("DS:{}:COMPUTE:{rpn}", name.name),
        }
    }

    /// Returns the `DS:...` arg
    fn as_arg_string(&self) -> String {
        self.arg.clone()
    }
}

/// A plain data source name, or a mapping referencing a `source` DS.
pub struct DataSourceName {
    /// The `name` string to use in a DS arg for `create`.
    name: String,
}

impl DataSourceName {
    /// A data source name that does not reference a source RRD DS.
    ///
    /// # Errors
    /// Returns `InvalidArgument` if the name is empty or contains `:`, which this wrapper uses
    /// as an argument separator.
    pub fn new(name: impl Into<String>) -> Result<Self, InvalidArgument> {
        let name = name.into();
        validate_ds_name(&name)?;
        Ok(Self { name })
    }

    /// A data source name that will be pre-filled from `src_ds_name`, optionally at source `index`.
    ///
    /// # Errors
    /// Returns `InvalidArgument` if either DS name is empty or contains a character this wrapper
    /// uses to compose mapping arguments.
    pub fn mapped(
        name: &str,
        src_ds_name: &str,
        index: Option<u32>,
    ) -> Result<Self, InvalidArgument> {
        validate_mapped_ds_name(name)?;
        validate_mapped_ds_name(src_ds_name)?;
        Ok(Self {
            name: match index {
                None => format!("{name}={src_ds_name}"),
                Some(i) => format!("{name}={src_ds_name}[{i}]"),
            },
        })
    }
}

fn validate_ds_name(name: &str) -> Result<(), InvalidArgument> {
    if !name.is_empty() && !name.contains(':') {
        Ok(())
    } else {
        Err(InvalidArgument(
            "Data source name must be non-empty and not contain ':'",
        ))
    }
}

fn validate_mapped_ds_name(name: &str) -> Result<(), InvalidArgument> {
    validate_ds_name(name)?;
    if name.contains(['=', '[', ']']) {
        Err(InvalidArgument(
            "Mapped data source names must not contain '=', '[' or ']'",
        ))
    } else {
        Ok(())
    }
}

/// Definition of an RRA to include in a new RRD.
pub struct Archive {
    consolidation_fn: ConsolidationFn,
    /// In `[0, 1]`
    xfiles_factor: f64,
    steps: u32,
    rows: u32,
}

impl Archive {
    /// `xfiles_factor` must be between 0 and 1.
    ///
    /// Returns `Some` if `xfiles_factor` is valid, `None` otherwise.
    ///
    /// # Errors
    /// Returns `InvalidArgument` if `xfiles_factor` is not in `[0, 1)`.
    pub fn new(
        consolidation_fn: ConsolidationFn,
        xfiles_factor: f64,
        steps: u32,
        rows: u32,
    ) -> Result<Self, InvalidArgument> {
        // documented as inclusive, but rrdcreate rejects 1.0
        if (0.0_f64..1.0_f64).contains(&xfiles_factor) {
            Ok(Self {
                consolidation_fn,
                xfiles_factor,
                steps,
                rows,
            })
        } else {
            Err(InvalidArgument("xfiles_factor must be in [0, 1]"))
        }
    }
}

impl Archive {
    /// Returns `RRA:...`
    fn as_arg_string(&self) -> String {
        format!(
            "RRA:{}:{}:{}:{}",
            self.consolidation_fn.as_arg_str(),
            self.xfiles_factor,
            self.steps,
            self.rows
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ds_name_valid() {
        assert!(DataSourceName::new("foo_123").is_ok());
        assert!(DataSourceName::new("a1234567890123456789").is_ok());
        assert!(DataSourceName::new("foo-bar").is_ok());
    }

    #[test]
    fn ds_name_invalid() {
        assert!(DataSourceName::new("").is_err());
        assert!(DataSourceName::new("foo:bar").is_err());
    }

    #[test]
    fn mapped_ds_name_validates_parts() {
        assert_eq!(
            "dst=src[2]",
            DataSourceName::mapped("dst", "src", Some(2)).unwrap().name
        );
        assert!(DataSourceName::mapped("dst-name", "src", None).is_ok());
        assert!(DataSourceName::mapped("dst", "src-name", None).is_ok());
        assert!(DataSourceName::mapped("dst=name", "src", None).is_err());
        assert!(DataSourceName::mapped("dst[1]", "src", None).is_err());
    }
}
