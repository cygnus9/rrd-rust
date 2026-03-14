//! Create new RRDs.

use crate::error::InvalidArgument;
use crate::{
    error::{return_code_to_result, RrdResult},
    util::{path_to_str, ArrayOfStrings, NullTerminatedArrayOfStrings},
    ConsolidationFn, Timestamp, TimestampExt,
};
use log::debug;
use std::{ffi::CString, path::Path, ptr::null, time::Duration};

/// Create a new RRD.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdcreate.en.html>.
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
        .map(|p| path_to_str(p).and_then(|s| CString::new(s).map_err(|e| e.into())))
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

    debug!(
        "Create: file={filename:?} start={} step={} no_overwrite={no_overwrite} template={template:?} sources={sources:?} args={args:?}",
        start.as_time_t(),
        step.as_secs()
    );

    let rc = unsafe {
        rrd_sys::rrd_create_r2(
            filename.as_ptr(),
            #[allow(clippy::useless_conversion)]
            // windows c_ulong is u32
            step.as_secs().try_into().expect("step too big for c_ulong"),
            start.as_time_t(),
            no_overwrite.into(),
            sources.as_ptr(),
            template.map_or_else(null, |s| s.as_ptr()),
            args.len()
                .try_into()
                .expect("Too many args to fit in rrd_int"),
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
    pub fn gauge(name: DataSourceName, heartbeat: u32, min: Option<f64>, max: Option<f64>) -> Self {
        Self {
            arg: format!(
                "DS:{}:GAUGE:{heartbeat}:{}:{}",
                name.name,
                min.map(|m| m.to_string())
                    .unwrap_or_else(|| "U".to_string()),
                max.map(|m| m.to_string())
                    .unwrap_or_else(|| "U".to_string())
            ),
        }
    }

    /// Define a 'COUNTER` data source.
    pub fn counter(
        name: DataSourceName,
        heartbeat: u32,
        min: Option<u64>,
        max: Option<u64>,
    ) -> Self {
        Self {
            arg: format!(
                "DS:{}:COUNTER:{heartbeat}:{}:{}",
                name.name,
                min.map(|m| m.to_string())
                    .unwrap_or_else(|| "U".to_string()),
                max.map(|m| m.to_string())
                    .unwrap_or_else(|| "U".to_string())
            ),
        }
    }

    /// Define a 'DCOUNTER` data source.
    pub fn dcounter(
        name: DataSourceName,
        heartbeat: u32,
        min: Option<f64>,
        max: Option<f64>,
    ) -> Self {
        Self {
            arg: format!(
                "DS:{}:DCOUNTER:{heartbeat}:{}:{}",
                name.name,
                min.map(|m| m.to_string())
                    .unwrap_or_else(|| "U".to_string()),
                max.map(|m| m.to_string())
                    .unwrap_or_else(|| "U".to_string())
            ),
        }
    }

    /// Define a 'DERIVE` data source.
    pub fn derive(
        name: DataSourceName,
        heartbeat: u32,
        min: Option<u64>,
        max: Option<u64>,
    ) -> Self {
        Self {
            arg: format!(
                "DS:{}:DERIVE:{heartbeat}:{}:{}",
                name.name,
                min.map(|m| m.to_string())
                    .unwrap_or_else(|| "U".to_string()),
                max.map(|m| m.to_string())
                    .unwrap_or_else(|| "U".to_string())
            ),
        }
    }

    /// Define a 'DDERIVE` data source.
    pub fn dderive(
        name: DataSourceName,
        heartbeat: u32,
        min: Option<f64>,
        max: Option<f64>,
    ) -> Self {
        Self {
            arg: format!(
                "DS:{}:DDERIVE:{heartbeat}:{}:{}",
                name.name,
                min.map(|m| m.to_string())
                    .unwrap_or_else(|| "U".to_string()),
                max.map(|m| m.to_string())
                    .unwrap_or_else(|| "U".to_string())
            ),
        }
    }

    /// Define an 'ABSOLUTE` data source.
    pub fn absolute(
        name: DataSourceName,
        heartbeat: u32,
        min: Option<u64>,
        max: Option<u64>,
    ) -> Self {
        Self {
            arg: format!(
                "DS:{}:ABSOLUTE:{heartbeat}:{}:{}",
                name.name,
                min.map(|m| m.to_string())
                    .unwrap_or_else(|| "U".to_string()),
                max.map(|m| m.to_string())
                    .unwrap_or_else(|| "U".to_string())
            ),
        }
    }

    /// Define a 'COMPUTE` data source.
    pub fn compute(name: DataSourceName, rpn: &str) -> Self {
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
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// A data source name that will be pre-filled from `src_ds_name`, optionally at source `index`.
    pub fn mapped(name: &str, src_ds_name: &str, index: Option<u32>) -> Self {
        Self {
            name: match index {
                None => format!("{name}={src_ds_name}"),
                Some(i) => format!("{name}={src_ds_name}[{i}]"),
            },
        }
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
