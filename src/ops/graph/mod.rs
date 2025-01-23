//! Render graphs from RRD data.
//!
//! There are many options for graphs. See <https://oss.oetiker.ch/rrdtool/doc/rrdgraph.en.html> and
//! <https://oss.oetiker.ch/rrdtool/tut/rrdtutorial.en.html> for more detail.
pub mod elements;
pub mod props;

use crate::error::InvalidArgument;
use crate::{
    error::{get_rrd_error, RrdError, RrdResult},
    ops::{
        graph::{
            elements::GraphElement,
            props::{GraphProps, ImageFormat},
        },
        info::{self, InfoValue},
    },
    util::ArrayOfStrings,
    Timestamp,
};
use log::debug;
use nom::{bytes, character::complete, combinator, sequence, Finish};
use std::{collections, ffi::CString, fmt::Write as _};

/// Returns a tuple containing the graph image data in the specified format and metadata about the
/// graph.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdgraph.en.html> or `/tests/tutorial.rs`.
pub fn graph(
    image_format: ImageFormat,
    props: GraphProps,
    elements: &[GraphElement],
) -> RrdResult<(Vec<u8>, GraphMetadata)> {
    // detect error conditions that will confusingly produce no librrd output whatsoever
    if !elements.iter().any(|c| matches!(c, GraphElement::Def(_))) {
        return Err(RrdError::InvalidArgument(
            "Must have at least one Def element".to_string(),
        ));
    }
    if !elements.iter().any(|c| {
        matches!(
            c,
            GraphElement::Print(_)
                | GraphElement::GPrint(_)
                | GraphElement::Line(_)
                | GraphElement::Area(_)
        )
    }) {
        return Err(RrdError::InvalidArgument(
            "Must have at least one Line, Area, GPrint, or Print element".to_string(),
        ));
    }

    // Need to include initial "graphv" command since that's how `rrdtool` invokes rrd_graph_v.
    // Filename `-` means include image data in the return hash rather than writing to a file
    let mut args = vec!["graphv".to_string(), "-".to_string()];
    image_format.append_to(&mut args)?;
    props.append_to(&mut args)?;
    for c in elements {
        c.append_to(&mut args)?;
    }

    debug!("Graph: args={args:?}");
    let args = args
        .into_iter()
        .map(CString::new)
        .collect::<Result<ArrayOfStrings, _>>()?;

    let info_ptr = unsafe {
        rrd_sys::rrd_graph_v(
            args.len().try_into().expect("Implausibly huge argc"),
            // different librrd versions differ in mutability of this pointer
            args.as_ptr() as _,
        )
    };
    if info_ptr.is_null() {
        return Err(get_rrd_error().unwrap_or_else(|| {
            RrdError::Internal("No graph data produced, but no librrd error".to_string())
        }));
    }

    let mut info = info::build_info_map(info_ptr);

    // pull out image first so debug output isn't massive
    let image = extract_info_value(&mut info, "image", |v| v.into_blob())?;

    debug!("Graph output: {info:?}");

    let graph_left = extract_info_value(&mut info, "graph_left", |v| v.into_count())?;
    let graph_top = extract_info_value(&mut info, "graph_top", |v| v.into_count())?;
    let graph_width = extract_info_value(&mut info, "graph_width", |v| v.into_count())?;
    let graph_height = extract_info_value(&mut info, "graph_height", |v| v.into_count())?;
    let image_width = extract_info_value(&mut info, "image_width", |v| v.into_count())?;
    let image_height = extract_info_value(&mut info, "image_height", |v| v.into_count())?;
    let graph_start =
        extract_info_value(&mut info, "graph_start", |v| v.into_count()).map(|t| {
            Timestamp::from_timestamp(t.try_into().expect("Graph start overflow"), 0)
                .expect("Impossible graph start")
        })?;
    let graph_end = extract_info_value(&mut info, "graph_end", |v| v.into_count()).map(|t| {
        Timestamp::from_timestamp(t.try_into().expect("Graph end overflow"), 0)
            .expect("Impossible graph end")
    })?;
    let value_min = extract_info_value(&mut info, "value_min", |v| v.into_value())?;
    let value_max = extract_info_value(&mut info, "value_max", |v| v.into_value())?;

    Ok((
        image,
        GraphMetadata {
            graph_left,
            graph_top,
            graph_width,
            graph_height,
            graph_start,
            graph_end,
            image_width,
            image_height,
            value_min,
            value_max,
            extra_info: info,
        },
    ))
}

/// Metadata about a rendered graph.
///
/// See [`graph`].
#[derive(Clone, Debug, PartialEq)]
pub struct GraphMetadata {
    /// Offset in pixels from the left edge of the image
    pub graph_left: u64,
    /// Offset in pixels from the top edge of the image
    pub graph_top: u64,
    /// Width in pixels of the graph in the image
    pub graph_width: u64,
    /// Height in pixels of the graph in the image
    pub graph_height: u64,
    /// Time at the start of the graph
    pub graph_start: Timestamp,
    /// Time at the end of the graph
    pub graph_end: Timestamp,
    /// Width in pixels
    pub image_width: u64,
    /// Height in pixels
    pub image_height: u64,
    /// Min value in the graph
    pub value_min: f64,
    /// Max value in the graph
    pub value_max: f64,
    /// Additional data returned from `rrd_graph_v`.
    ///
    /// Contents depend on the commands given.
    pub extra_info: collections::HashMap<String, InfoValue>,
}

/// RGB(A) color.
///
/// # Examples
///
/// `Color` can be parsed from a CSS-style 6 or 8 digit hex RGB(A) string.
///
/// RGB, no alpha:
///
/// ```
/// use rrd::ops::graph::Color;
/// let color: Color = "#012345".parse().unwrap();
/// assert_eq!(None, color.alpha);
/// ```
///
/// RGBA:
///
/// ```
/// use rrd::ops::graph::Color;
/// let color: Color = "#01234567".parse().unwrap();
/// assert_eq!(Some(0x67), color.alpha);
/// ```
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdgraph.en.html>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: Option<u8>,
}

impl Color {
    /// Appends `#hex`.
    fn append_to(&self, s: &mut String) {
        match self.alpha {
            None => write!(s, "#{:02X}{:02X}{:02X}", self.red, self.green, self.blue,),
            Some(alpha) => write!(
                s,
                "#{:02X}{:02X}{:02X}{:02X}",
                self.red, self.green, self.blue, alpha
            ),
        }
        .unwrap()
    }
}

impl std::str::FromStr for Color {
    type Err = InvalidArgument;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        combinator::map(
            combinator::all_consuming(sequence::preceded(
                bytes::complete::tag("#"),
                sequence::tuple((
                    parse_hex_byte,
                    parse_hex_byte,
                    parse_hex_byte,
                    combinator::opt(parse_hex_byte),
                )),
            )),
            |(red, green, blue, alpha)| Color {
                red,
                green,
                blue,
                alpha,
            },
        )(s)
        .finish()
        .map_err(|_| InvalidArgument("Invalid color"))
        .map(|(_rem, c)| c)
    }
}

/// Incrementally build up the args to use in a graph invocation.
trait AppendArgs {
    /// Append suitable args to the args buffer.
    ///
    /// Returns Result to allow users to specify a PathBuf which may later fail conversion.
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()>;
}

fn extract_info_value<T>(
    info: &mut collections::HashMap<String, InfoValue>,
    key: &str,
    transform: impl FnOnce(InfoValue) -> Option<T>,
) -> RrdResult<T> {
    let value = info
        .remove(key)
        .ok_or_else(|| RrdError::Internal(format!("Graph info: no {key}")))?;
    transform(value)
        .ok_or_else(|| RrdError::Internal(format!("Graph info: unexpected {key} value type")))
}

fn parse_hex_byte(input: &str) -> nom::IResult<&str, u8> {
    combinator::map_opt(
        sequence::pair(complete::anychar, complete::anychar),
        |(hi, lo)| {
            let hi = hi.to_digit(16)? as u8;
            let lo = lo.to_digit(16)? as u8;

            Some((hi << 4) | lo)
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_color_no_alpha() {
        assert_eq!(
            Color {
                red: 0x01,
                green: 0x23,
                blue: 0x45,
                alpha: None,
            },
            "#012345".parse().unwrap()
        )
    }

    #[test]
    fn parse_color_with_alpha() {
        assert_eq!(
            Color {
                red: 0x01,
                green: 0x23,
                blue: 0x45,
                alpha: Some(0x67),
            },
            "#01234567".parse().unwrap()
        )
    }

    #[test]
    fn parse_color_err_invalid_hex() {
        assert!("#0000ZZ".parse::<Color>().is_err());
    }

    #[test]
    fn parse_color_err_no_prefix() {
        assert!("FFFFFF".parse::<Color>().is_err());
    }

    #[test]
    fn parse_color_err_wrong_length() {
        // too short
        assert!("#FFFFF".parse::<Color>().is_err());
        // in between rgb and rgba
        assert!("#FFFFFFF".parse::<Color>().is_err());
        // too long
        assert!("#FFFFFFFFF".parse::<Color>().is_err());
    }
}
