//! The data and visual elements to include in the graph.
//!
//! Extracting and processing data:
//!
//! - [`Def`]
//! - [`VDef`]
//! - [`CDef`]
//! - See <https://oss.oetiker.ch/rrdtool/doc/rrdgraph_data.en.html>
//!
//! Visual elements and controls:
//!
//! - [`Print`]
//! - [`GPrint`]
//! - [`Comment`]
//! - [`VRule`]
//! - [`HRule`]
//! - [`Line`]
//! - [`Area`]
//! - [`Tick`]
//! - [`Shift`]
//! - [`TextAlign`]
//! - See <https://oss.oetiker.ch/rrdtool/doc/rrdgraph_graph.en.html>

use crate::{
    error::{InvalidArgument, RrdResult},
    ops::graph::{AppendArgs, Color},
    util::path_to_str,
    ConsolidationFn, Timestamp,
};
use itertools::Itertools;
use std::{fmt::Write as _, path::PathBuf, sync};

/// Enum expressing all possible elements.
///
/// This is typically not used directly, as it only exists as a convenience type to be able to
/// `.into()` other elements ([`Def`], etc) into a common type in a `graph()` call.
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum GraphElement {
    Def(Def),
    CDef(CDef),
    VDef(VDef),
    Print(Print),
    GPrint(GPrint),
    Comment(Comment),
    VRule(VRule),
    HRule(HRule),
    Line(Line),
    Area(Area),
    Tick(Tick),
    Shift(Shift),
    TextAlign(TextAlign),
}

impl AppendArgs for GraphElement {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        match self {
            GraphElement::Def(c) => c.append_to(args),
            GraphElement::CDef(c) => c.append_to(args),
            GraphElement::VDef(c) => c.append_to(args),
            GraphElement::Print(c) => c.append_to(args),
            GraphElement::GPrint(c) => c.append_to(args),
            GraphElement::Comment(c) => c.append_to(args),
            GraphElement::VRule(c) => c.append_to(args),
            GraphElement::HRule(c) => c.append_to(args),
            GraphElement::Line(c) => c.append_to(args),
            GraphElement::Area(c) => c.append_to(args),
            GraphElement::Tick(c) => c.append_to(args),
            GraphElement::Shift(c) => c.append_to(args),
            GraphElement::TextAlign(c) => c.append_to(args),
        }
    }
}

/// Define data to fetch from a DS.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdgraph_data.en.html>
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct Def {
    pub var_name: VarName,
    pub rrd: PathBuf,
    pub ds_name: String,
    pub consolidation_fn: ConsolidationFn,
    pub step: Option<u32>,
    pub start: Option<Timestamp>,
    pub end: Option<Timestamp>,
    pub reduce: Option<ConsolidationFn>,
    // skipping daemon as we do not support that,
}

impl AppendArgs for Def {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        let mut s = format!(
            "DEF:{}={}:{}:{}",
            self.var_name.name,
            path_to_str(&self.rrd)?,
            self.ds_name,
            self.consolidation_fn.as_arg_str(),
        );

        if let Some(step) = self.step {
            write!(s, ":step={step}").unwrap();
        }
        if let Some(start) = self.start {
            write!(s, ":start={}", start.timestamp()).unwrap();
        }
        if let Some(end) = self.end {
            write!(s, ":end={}", end.timestamp()).unwrap();
        }
        if let Some(reduce) = self.reduce {
            write!(s, ":reduce={}", reduce.as_arg_str()).unwrap();
        }

        args.push(s);
        Ok(())
    }
}

impl From<Def> for GraphElement {
    fn from(value: Def) -> Self {
        Self::Def(value)
    }
}

/// RPN to produce a value and/or time.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdgraph_data.en.html>
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct VDef {
    pub var_name: VarName,
    pub rpn: String,
}
impl AppendArgs for VDef {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        args.push(format!("VDEF:{}={}", self.var_name.name, self.rpn));
        Ok(())
    }
}

impl From<VDef> for GraphElement {
    fn from(value: VDef) -> Self {
        Self::VDef(value)
    }
}

/// RPN to produce a new set of data points.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdgraph_data.en.html>

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct CDef {
    pub var_name: VarName,
    pub rpn: String,
}

impl AppendArgs for CDef {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        args.push(format!("CDEF:{}={}", self.var_name.name, self.rpn));
        Ok(())
    }
}

impl From<CDef> for GraphElement {
    fn from(value: CDef) -> Self {
        Self::CDef(value)
    }
}

/// A variable name.
///
/// To avoid clashing with RPN operators, don't use `UPPERCASE`.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdgraph_data.en.html>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarName {
    name: String,
}

static VALID_VNAME: sync::LazyLock<regex::Regex> =
    sync::LazyLock::new(|| regex::Regex::new("^[A-Za-z0-9_-]+$").unwrap());

impl VarName {
    /// Create a new VarName, if the provided string is a valid name.
    pub fn new(name: impl Into<String>) -> Result<Self, InvalidArgument> {
        let s = name.into();
        if s.len() <= 255 && VALID_VNAME.is_match(&s) {
            Ok(Self { name: s })
        } else {
            Err(InvalidArgument("Invalid var name"))
        }
    }
}

impl TryFrom<String> for VarName {
    type Error = InvalidArgument;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
impl TryFrom<&str> for VarName {
    type Error = InvalidArgument;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.to_string())
    }
}

/// Specify text to print on the graph.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdgraph_graph.en.html>
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct Print {
    /// Must be a var name defined by a [`VDef`].
    pub var_name: VarName,
    pub format: String,
    pub format_mode: Option<PrintFormatMode>,
}

impl AppendArgs for Print {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        let fmt_mode = match &self.format_mode {
            None => "".to_string(),
            Some(fm) => {
                format!(
                    ":{}",
                    match fm {
                        PrintFormatMode::StrfTime => "strftime",
                        PrintFormatMode::ValStrfTime => "valstrftime",
                        PrintFormatMode::ValStrfDuration => "valstrfduration",
                    }
                )
            }
        };
        args.push(format!(
            "PRINT:{}:{}{fmt_mode}",
            self.var_name.name, self.format
        ));
        Ok(())
    }
}

impl From<Print> for GraphElement {
    fn from(value: Print) -> Self {
        Self::Print(value)
    }
}

/// See [`Print`].
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdgraph_graph.en.html>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum PrintFormatMode {
    StrfTime,
    ValStrfTime,
    ValStrfDuration,
}

/// Like [`Print`] but inside the graph.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdgraph_graph.en.html>
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct GPrint {
    pub var_name: VarName,
    pub format: String,
}

impl AppendArgs for GPrint {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        args.push(format!("GPRINT:{}:{}", self.var_name.name, self.format));
        Ok(())
    }
}

impl From<GPrint> for GraphElement {
    fn from(value: GPrint) -> Self {
        Self::GPrint(value)
    }
}

/// Text to include in the legend.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdgraph_graph.en.html>
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct Comment {
    pub text: String,
}

impl AppendArgs for Comment {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        args.push(format!("COMMENT:{}", self.text));
        Ok(())
    }
}

impl From<Comment> for GraphElement {
    fn from(value: Comment) -> Self {
        Self::Comment(value)
    }
}

/// A vertical line at a specific time.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdgraph_graph.en.html>
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct VRule {
    pub value: Value,
    pub color: Color,
    pub legend: Option<Legend>,
    pub dashes: Option<Dashes>,
}

impl AppendArgs for VRule {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        let mut s = "VRULE:".to_string();
        self.value.append_to(&mut s);
        self.color.append_to(&mut s);
        if let Some(l) = &self.legend {
            l.append_to(&mut s);
        }
        if let Some(d) = &self.dashes {
            d.append_to(&mut s);
        };
        args.push(s);
        Ok(())
    }
}

impl From<VRule> for GraphElement {
    fn from(value: VRule) -> Self {
        Self::VRule(value)
    }
}

/// A var reference, timestamp, or fixed value.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdgraph_graph.en.html>
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum Value {
    Variable(VarName),
    Timestamp(Timestamp),
    Constant(f64),
}

impl Value {
    fn append_to(&self, s: &mut String) {
        match self {
            Value::Variable(v) => write!(s, "{}", v.name),
            Value::Timestamp(t) => write!(s, "{}", t.timestamp()),
            Value::Constant(f) => {
                write!(s, "{}", f)
            }
        }
        .unwrap()
    }
}

impl From<VarName> for Value {
    fn from(value: VarName) -> Self {
        Self::Variable(value)
    }
}

impl From<Timestamp> for Value {
    fn from(value: Timestamp) -> Self {
        Self::Timestamp(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Constant(value)
    }
}

/// Dash configuration for a [`VRule`], [`HRule`], or [`Line`].
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdgraph_graph.en.html>
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct Dashes {
    pub spacing: Option<DashSpacing>,
    pub offset: Option<u32>,
}

impl Dashes {
    /// Returns `:dashes...`
    fn append_to(&self, s: &mut String) {
        let prefix = ":dashes";
        let spacing_str = match &self.spacing {
            None => String::new(),
            Some(spacing) => match spacing {
                DashSpacing::Simple(num) => format!("={num}"),
                DashSpacing::Custom(nums) => format!(
                    "={}",
                    nums.iter()
                        .flat_map(|(on, off)| [on, off].into_iter())
                        .join(",")
                ),
            },
        };
        let offset_str = self
            .offset
            .map(|o| format!(":dash-offset={o}"))
            .unwrap_or_default();

        write!(s, "{prefix}{spacing_str}{offset_str}").unwrap()
    }
}

/// Dash spacing.
///
/// See [`Dashes`] and <https://oss.oetiker.ch/rrdtool/doc/rrdgraph_graph.en.html>
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DashSpacing {
    /// Must be positive
    Simple(u32),
    /// Pairs of (on, off) spacing.
    ///
    /// Must be positive.
    Custom(Vec<(u32, u32)>),
}

/// A horizontal line at a particular value.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdgraph_graph.en.html>
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct HRule {
    pub value: Value,
    pub color: Color,
    pub legend: Option<Legend>,
    pub dashes: Option<Dashes>,
}

impl AppendArgs for HRule {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        let mut s = "HRULE:".to_string();
        self.value.append_to(&mut s);
        self.color.append_to(&mut s);
        if let Some(l) = &self.legend {
            l.append_to(&mut s);
        }
        if let Some(d) = &self.dashes {
            d.append_to(&mut s);
        }
        args.push(s);
        Ok(())
    }
}

impl From<HRule> for GraphElement {
    fn from(value: HRule) -> Self {
        Self::HRule(value)
    }
}

/// Plot the value of a var over time.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdgraph_graph.en.html>
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct Line {
    pub width: f64,
    pub value: VarName,
    pub color: Option<ColorWithLegend<Color>>,
    pub stack: bool,
    pub skip_scale: bool,
    pub dashes: Option<Dashes>,
}

impl AppendArgs for Line {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        let mut s = format!("LINE{}:{}", self.width, self.value.name);

        if let Some(cwl) = &self.color {
            cwl.color.append_to(&mut s);
            if let Some(l) = &cwl.legend {
                l.append_to(&mut s);
            }
        }

        // If no color, docs say `LINEx:value::STACK`
        if self.stack {
            if self.color.is_none() {
                s.push(':');
            }
            s.push_str(":STACK")
        }
        if self.skip_scale {
            s.push_str(":skipscale")
        }
        if let Some(d) = &self.dashes {
            d.append_to(&mut s);
        }
        args.push(s);
        Ok(())
    }
}

impl From<Line> for GraphElement {
    fn from(value: Line) -> Self {
        Self::Line(value)
    }
}

/// For [`Line`] and [`Area`], legend can only be specified if a color is set.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdgraph_graph.en.html>
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct ColorWithLegend<C> {
    pub color: C,
    pub legend: Option<Legend>,
}

/// Like [`Line`], but with the area between the x axis and the line filled in.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdgraph_graph.en.html>
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct Area {
    pub value: VarName,
    pub color: Option<ColorWithLegend<AreaColor>>,
    pub stack: bool,
    pub skip_scale: bool,
}

impl AppendArgs for Area {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        let mut s = format!("AREA:{}", self.value.name);

        let grad_height = if let Some(cwl) = &self.color {
            let gh = match cwl.color {
                AreaColor::Color(c) => {
                    c.append_to(&mut s);
                    None
                }
                AreaColor::Gradient {
                    color1,
                    color2,
                    gradient_height,
                } => {
                    color1.append_to(&mut s);
                    color2.append_to(&mut s);
                    gradient_height
                }
            };

            if let Some(l) = &cwl.legend {
                l.append_to(&mut s);
            }

            gh
        } else {
            None
        };

        // If no color, docs imply `AREA:value::STACK` by saying it should be like LINE
        if self.stack {
            if self
                .color
                .as_ref()
                .map(|c| c.legend.is_none())
                .unwrap_or(true)
            {
                s.push(':');
            }
            s.push_str(":STACK")
        }
        if self.skip_scale {
            s.push_str(":skipscale")
        }

        if let Some(gh) = grad_height {
            write!(s, ":gradheight={gh}").unwrap()
        }
        args.push(s);
        Ok(())
    }
}

impl From<Area> for GraphElement {
    fn from(value: Area) -> Self {
        Self::Area(value)
    }
}

/// Color or gradient for an [`Area`].
///
/// See [`Area`] and <https://oss.oetiker.ch/rrdtool/doc/rrdgraph_graph.en.html>
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum AreaColor {
    Color(Color),
    Gradient {
        color1: Color,
        color2: Color,
        gradient_height: Option<f64>,
    },
}

/// Draw tick marks for nonzero values.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdgraph_graph.en.html>
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct Tick {
    pub var_name: VarName,
    pub color: Color,
    pub fraction: Option<f64>,
    pub legend: Option<Legend>,
}

impl AppendArgs for Tick {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        let mut s = format!("TICK:{}", self.var_name.name);
        self.color.append_to(&mut s);
        if let Some(f) = self.fraction {
            write!(s, ":{f}").unwrap();
        }
        if let Some(l) = &self.legend {
            l.append_to(&mut s);
        }
        args.push(s);
        Ok(())
    }
}

impl From<Tick> for GraphElement {
    fn from(value: Tick) -> Self {
        Self::Tick(value)
    }
}

/// Shift the offset for subsequent elements.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdgraph_graph.en.html>
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct Shift {
    pub var_name: VarName,
    pub offset: Offset,
}

impl AppendArgs for Shift {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        let mut s = format!("SHIFT:{}:", self.var_name.name,);
        self.offset.append_to(&mut s);
        args.push(s);
        Ok(())
    }
}

impl From<Shift> for GraphElement {
    fn from(value: Shift) -> Self {
        Self::Shift(value)
    }
}

/// An offset used in [`Shift`].
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdgraph_graph.en.html>
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum Offset {
    Variable(VarName),
    TimeDelta(f64),
}

impl Offset {
    fn append_to(&self, s: &mut String) {
        match self {
            Offset::Variable(v) => write!(s, "{}", v.name),
            Offset::TimeDelta(t) => write!(s, "{t}"),
        }
        .unwrap()
    }
}

/// Controls text alignment for labels.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdgraph_graph.en.html>
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum TextAlign {
    Left,
    Right,
    Justified,
    Center,
}

impl AppendArgs for TextAlign {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        args.push(format!(
            "TEXTALIGN:{}",
            match self {
                TextAlign::Left => "left",
                TextAlign::Right => "right",
                TextAlign::Justified => "justified",
                TextAlign::Center => "center",
            }
        ));
        Ok(())
    }
}

impl From<TextAlign> for GraphElement {
    fn from(value: TextAlign) -> Self {
        Self::TextAlign(value)
    }
}

// TODO escape colons for the user
/// Text to include in the legend for the containing element.
///
/// Colons (`:`) must be escaped as `\:`, which in a string literal needs the backslash escaped
/// as well, so it would be typed `"\\:"`.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdgraph_graph.en.html>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Legend(String);

impl Legend {
    /// Appends `:` followed by quote-wrapped legend text.
    fn append_to(&self, s: &mut String) {
        // It's unclear from the docs -- does this need to be quoted, or is that only to deal with
        // shell command parsing?
        write!(s, ":{}", self.0).unwrap()
    }
}

impl<S: Into<String>> From<S> for Legend {
    fn from(value: S) -> Self {
        Self(value.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vname_regex_valid() {
        assert!(VALID_VNAME.is_match("foo_bar-baz-1"));
    }

    #[test]
    fn vname_regex_invalid() {
        assert!(!VALID_VNAME.is_match("foo@bar"));
    }

    #[test]
    fn def() {
        let mut args = vec![];
        Def {
            var_name: VarName::new("var".to_string()).unwrap(),
            rrd: "data.rrd".into(),
            ds_name: "DS1".to_string(),
            consolidation_fn: ConsolidationFn::Avg,
            step: Some(1),
            start: Some(chrono::DateTime::from_timestamp(100, 0).unwrap()),
            end: Some(chrono::DateTime::from_timestamp(1000, 0).unwrap()),
            reduce: Some(ConsolidationFn::Max),
        }
        .append_to(&mut args)
        .unwrap();

        let expected = ["DEF:var=data.rrd:DS1:AVERAGE:step=1:start=100:end=1000:reduce=MAX"];
        assert_eq!(
            expected.into_iter().map(|s| s.to_string()).collect_vec(),
            args
        );
    }
    #[test]
    fn vdef() {
        let mut args = vec![];
        VDef {
            var_name: VarName::new("var".to_string()).unwrap(),
            rpn: "rpn".to_string(),
        }
        .append_to(&mut args)
        .unwrap();

        let expected = ["VDEF:var=rpn"];
        assert_eq!(
            expected.into_iter().map(|s| s.to_string()).collect_vec(),
            args
        );
    }
    #[test]
    fn cdef() {
        let mut args = vec![];
        CDef {
            var_name: VarName::new("var".to_string()).unwrap(),
            rpn: "rpn".to_string(),
        }
        .append_to(&mut args)
        .unwrap();

        let expected = ["CDEF:var=rpn"];
        assert_eq!(
            expected.into_iter().map(|s| s.to_string()).collect_vec(),
            args
        );
    }
    #[test]
    fn print() {
        let mut args = vec![];
        Print {
            var_name: VarName::new("var".to_string()).unwrap(),
            format: "fmt".into(),
            format_mode: Some(PrintFormatMode::ValStrfTime),
        }
        .append_to(&mut args)
        .unwrap();

        let expected = ["PRINT:var:fmt:valstrftime"];
        assert_eq!(
            expected.into_iter().map(|s| s.to_string()).collect_vec(),
            args
        );
    }
    #[test]
    fn gprint() {
        let mut args = vec![];
        GPrint {
            var_name: VarName::new("var".to_string()).unwrap(),
            format: "fmt".into(),
        }
        .append_to(&mut args)
        .unwrap();

        let expected = ["GPRINT:var:fmt"];
        assert_eq!(
            expected.into_iter().map(|s| s.to_string()).collect_vec(),
            args
        );
    }
    #[test]
    fn comment() {
        let mut args = vec![];
        Comment {
            text: "comment".into(),
        }
        .append_to(&mut args)
        .unwrap();

        let expected = ["COMMENT:comment"];
        assert_eq!(
            expected.into_iter().map(|s| s.to_string()).collect_vec(),
            args
        );
    }
    #[test]
    fn vrule() {
        let mut args = vec![];
        VRule {
            value: Value::Variable(VarName::new("var").unwrap()),
            color: "#01020304".parse().unwrap(),
            legend: Some("foo".to_string().into()),
            dashes: Some(Dashes {
                spacing: Some(DashSpacing::Simple(4)),
                offset: Some(10),
            }),
        }
        .append_to(&mut args)
        .unwrap();

        let expected = ["VRULE:var#01020304:foo:dashes=4:dash-offset=10"];
        assert_eq!(
            expected.into_iter().map(|s| s.to_string()).collect_vec(),
            args
        );
    }
    #[test]
    fn hrule() {
        let mut args = vec![];
        HRule {
            value: Value::Timestamp(Timestamp::from_timestamp(1000, 0).unwrap()),
            color: "#010203".parse().unwrap(),
            legend: None,
            dashes: Some(Dashes {
                spacing: Some(DashSpacing::Custom(vec![(1, 2), (3, 4)])),
                offset: None,
            }),
        }
        .append_to(&mut args)
        .unwrap();

        let expected = ["HRULE:1000#010203:dashes=1,2,3,4"];
        assert_eq!(
            expected.into_iter().map(|s| s.to_string()).collect_vec(),
            args
        );
    }
    #[test]
    fn line() {
        let mut args = vec![];
        Line {
            width: 3.2,
            value: VarName::new("var").unwrap(),
            color: Some(ColorWithLegend {
                color: "#01020304".parse().unwrap(),
                legend: Some("foo".to_string().into()),
            }),
            stack: true,
            skip_scale: true,
            dashes: None,
        }
        .append_to(&mut args)
        .unwrap();

        let expected = ["LINE3.2:var#01020304:foo:STACK:skipscale"];
        assert_eq!(
            expected.into_iter().map(|s| s.to_string()).collect_vec(),
            args
        );
    }
    #[test]
    fn area() {
        let mut args = vec![];
        Area {
            value: VarName::new("var").unwrap(),
            color: Some(ColorWithLegend {
                color: AreaColor::Gradient {
                    color1: "#01020304".parse().unwrap(),
                    color2: "#41424344".parse().unwrap(),
                    gradient_height: Some(10.1),
                },
                legend: None,
            }),
            stack: true,
            skip_scale: true,
        }
        .append_to(&mut args)
        .unwrap();

        let expected = ["AREA:var#01020304#41424344::STACK:skipscale:gradheight=10.1"];
        assert_eq!(
            expected.into_iter().map(|s| s.to_string()).collect_vec(),
            args
        );
    }
    #[test]
    fn tick() {
        let mut args = vec![];
        Tick {
            var_name: VarName::new("var").unwrap(),
            color: "#01020304".parse().unwrap(),
            fraction: Some(1.2),
            legend: None,
        }
        .append_to(&mut args)
        .unwrap();

        let expected = ["TICK:var#01020304:1.2"];
        assert_eq!(
            expected.into_iter().map(|s| s.to_string()).collect_vec(),
            args
        );
    }
    #[test]
    fn shift() {
        let mut args = vec![];
        Shift {
            var_name: VarName::new("var").unwrap(),
            offset: Offset::Variable(VarName::new("offset").unwrap()),
        }
        .append_to(&mut args)
        .unwrap();

        let expected = ["SHIFT:var:offset"];
        assert_eq!(
            expected.into_iter().map(|s| s.to_string()).collect_vec(),
            args
        );
    }
    #[test]
    fn textalign() {
        let mut args = vec![];
        TextAlign::Justified.append_to(&mut args).unwrap();

        let expected = ["TEXTALIGN:justified"];
        assert_eq!(
            expected.into_iter().map(|s| s.to_string()).collect_vec(),
            args
        );
    }
}
