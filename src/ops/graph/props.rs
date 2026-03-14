//! Basic characteristics of a graph.

use crate::error::{InvalidArgument, RrdResult};
use crate::ops::graph::Color;
use crate::TimestampExt;
use crate::{ops::graph::AppendArgs, Timestamp};
use std::collections;

/// Top level graph properties.
///
/// See <https://oss.oetiker.ch/rrdtool/doc/rrdgraph.en.html>.
///
/// # Examples
///
/// There are many fields, so use `Default` to fill in the uninteresting ones.
///
/// ```
/// use rrd::ops::graph::props::{Limits, GraphProps};
///
/// let props = GraphProps {
///     limits: Limits {
///         upper_limit: 100.0.into(),
///         ..Default::default()
///     },
///     ..Default::default()
/// };
/// ```
#[derive(Default, Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct GraphProps {
    pub time_range: TimeRange,
    pub labels: Labels,
    pub size: Size,
    pub limits: Limits,
    pub x_axis: XAxis,
    pub y_axis: YAxis,
    // option since it seems that there are some mandatory args if using right y axis
    pub right_y_axis: Option<RightYAxis>,
    pub legend: Legend,
    pub misc: Misc,
}

impl AppendArgs for GraphProps {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        self.time_range.append_to(args)?;
        self.labels.append_to(args)?;
        self.size.append_to(args)?;
        self.limits.append_to(args)?;
        self.x_axis.append_to(args)?;
        self.y_axis.append_to(args)?;
        if let Some(rya) = &self.right_y_axis {
            rya.append_to(args)?;
        }
        self.legend.append_to(args)?;
        self.misc.append_to(args)?;
        Ok(())
    }
}

/// Time range to graph over.
///
/// See [`GraphProps`]
#[derive(Default, Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct TimeRange {
    pub start: Option<Timestamp>,
    pub end: Option<Timestamp>,
    pub step_seconds: Option<u32>,
}

impl AppendArgs for TimeRange {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        if let Some(s) = &self.start {
            args.push("--start".to_string());
            args.push(format!("{}", s.as_time_t()));
        }
        if let Some(e) = &self.end {
            args.push("--end".to_string());
            args.push(format!("{}", e.as_time_t()));
        }
        if let Some(ss) = &self.step_seconds {
            args.push("--step".to_string());
            args.push(format!("{ss}"));
        }
        Ok(())
    }
}

/// Title & vertical label.
///
/// See [`GraphProps`]
#[derive(Default, Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct Labels {
    pub title: Option<String>,
    pub vertical_label: Option<String>,
}

impl AppendArgs for Labels {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        if let Some(t) = &self.title {
            args.push("--title".to_string());
            args.push(t.clone());
        }

        if let Some(vl) = &self.vertical_label {
            args.push("--vertical-label".to_string());
            args.push(vl.clone())
        }
        Ok(())
    }
}

/// Canvas/graph size.
///
/// See [`GraphProps`]
#[derive(Default, Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct Size {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub full_size_mode: bool,
    pub only_graph: bool,
}

impl AppendArgs for Size {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        if let Some(w) = self.width {
            args.push("--width".to_string());
            args.push(format!("{w}"));
        }
        if let Some(h) = self.height {
            args.push("--height".to_string());
            args.push(format!("{h}"));
        }

        if self.only_graph {
            args.push("--only-graph".to_string());
        }

        if self.full_size_mode {
            args.push("--full-size-mode".to_string());
        }

        Ok(())
    }
}

/// Data limits and scale.
///
/// See [`GraphProps`]
#[derive(Default, Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct Limits {
    pub upper_limit: Option<f64>,
    pub lower_limit: Option<f64>,
    pub rigid: bool,
    pub allow_shrink: bool,
    /// If `Some`, enables alt autoscale
    pub alt_autoscale: Option<AltAutoscale>,
    pub no_grid_fit: bool,
}

impl AppendArgs for Limits {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        if let Some(ul) = self.upper_limit {
            args.push("--upper-limit".to_string());
            args.push(format!("{ul}"));
        }
        if let Some(ll) = self.lower_limit {
            args.push("--lower-limit".to_string());
            args.push(format!("{ll}"));
        }

        if self.rigid {
            args.push("--rigid".to_string());
        }

        if self.allow_shrink {
            args.push("--allow-shrink".to_string());
        }

        if let Some(aa) = &self.alt_autoscale {
            args.push("--alt-autoscale".to_string());

            if let Some(min) = aa.alt_autoscale_min {
                args.push("--alt-autoscale-min".to_string());
                args.push(format!("{min}"));
            }

            if let Some(max) = aa.alt_autoscale_max {
                args.push("--alt-autoscale-max".to_string());
                args.push(format!("{max}"));
            }
        }

        if self.no_grid_fit {
            args.push("--no-gridfit".to_string());
        }

        Ok(())
    }
}

/// Limits for alternate autoscale algorithm.
///
/// See [`Limits`]
#[derive(Default, Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct AltAutoscale {
    pub alt_autoscale_min: Option<f64>,
    pub alt_autoscale_max: Option<f64>,
}

/// X axis format.
///
/// See [`GraphProps`]
#[derive(Default, Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct XAxis {
    pub grid: Option<XAxisGrid>,
    pub week_format: Option<String>,
}

impl AppendArgs for XAxis {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        if let Some(xag) = &self.grid {
            args.push("--x-grid".to_string());
            match xag {
                XAxisGrid::None => args.push("none".to_string()),
                XAxisGrid::Custom {
                    base_grid_time,
                    base_grid_step,
                    major_grid_time,
                    major_grid_step,
                    labels_time,
                    labels_step,
                    label_placement,
                    label_format,
                } => args.push(format!(
                    "{}:{base_grid_step}:{}:{major_grid_step}:{}:{labels_step}:{label_placement}:{label_format}",
                    base_grid_time.as_arg_str(),
                    major_grid_time.as_arg_str(),
                    labels_time.as_arg_str()
                )),
            }
        }

        if let Some(wf) = &self.week_format {
            args.push("--week-fmt".to_string());
            args.push(wf.clone())
        }

        Ok(())
    }
}

/// X axis grid format and label.
///
/// See [`GraphProps`]
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum XAxisGrid {
    None,
    Custom {
        base_grid_time: AxisGridTimeUnit,
        base_grid_step: u32,
        major_grid_time: AxisGridTimeUnit,
        major_grid_step: u32,
        labels_time: AxisGridTimeUnit,
        labels_step: u32,
        label_placement: u32,
        label_format: String,
    },
}

/// Time unit used in grid labels.
///
/// See [`GraphProps`]
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub enum AxisGridTimeUnit {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

impl AxisGridTimeUnit {
    fn as_arg_str(&self) -> &'static str {
        match self {
            AxisGridTimeUnit::Second => "SECOND",
            AxisGridTimeUnit::Minute => "MINUTE",
            AxisGridTimeUnit::Hour => "HOUR",
            AxisGridTimeUnit::Day => "DAY",
            AxisGridTimeUnit::Week => "WEEK",
            AxisGridTimeUnit::Month => "MONTH",
            AxisGridTimeUnit::Year => "YEAR",
        }
    }
}

/// Y axis format.
///
/// See [`GraphProps`]
#[derive(Default, Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct YAxis {
    pub grid: Option<YAxisGrid>,
    pub formatter: Option<YAxisFormatter>,
    pub format: Option<String>,
    pub alt_y_grid: bool,
    pub logarithmic: bool,
    pub units_exponent: Option<UnitsExponent>,
    pub units_length: Option<u8>,
    pub units: Option<Units>,
}

impl AppendArgs for YAxis {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        if let Some(yag) = &self.grid {
            args.push("--y-grid".to_string());
            match yag {
                YAxisGrid::None => args.push("none".to_string()),
                YAxisGrid::Custom {
                    grid_step,
                    label_factor,
                } => {
                    args.push(format!("{grid_step}:{label_factor}"));
                }
            }
        }

        if let Some(yaf) = self.formatter {
            args.push("--left-axis-formatter".to_string());
            yaf.append_to(args)?;
        }

        if let Some(f) = &self.format {
            args.push("--left-axis-format".to_string());
            args.push(f.clone());
        }

        if self.alt_y_grid {
            args.push("--alt-y-grid".to_string());
        }

        if self.logarithmic {
            args.push("--logarithmic".to_string());
        }

        if let Some(ue) = self.units_exponent {
            args.push("--units-exponent".to_string());
            args.push(format!("{}", ue.exp));
        }

        if let Some(ul) = self.units_length {
            args.push("--units-length".to_string());
            args.push(format!("{ul}"));
        }

        if let Some(u) = self.units {
            args.push("--units".to_string());
            args.push(
                match u {
                    Units::Si => "si",
                }
                .to_string(),
            );
        }

        Ok(())
    }
}

/// Y axis grid format.
///
/// See [`GraphProps`]
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub enum YAxisGrid {
    None,
    Custom { grid_step: u32, label_factor: u32 },
}

/// Axis value formatter.
///
/// See [`GraphProps`]
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub enum YAxisFormatter {
    Numeric,
    Timestamp,
    Duration,
}

impl AppendArgs for YAxisFormatter {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        args.push(
            (match self {
                YAxisFormatter::Numeric => "numeric",
                YAxisFormatter::Timestamp => "timestamp",
                YAxisFormatter::Duration => "duration",
            })
            .to_string(),
        );

        Ok(())
    }
}

/// Y axis value exponent.
///
/// See [`GraphProps`]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitsExponent {
    /// Must be a multiple of 3.
    pub exp: i8,
}

/// Y axis unit.
///
/// See [`GraphProps`]
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub enum Units {
    Si,
}

impl UnitsExponent {
    /// `exp` must be a multiple of `3` in `[-18, 8]`.
    pub fn new(exp: i8) -> Result<Self, InvalidArgument> {
        if (-18..=18).contains(&exp) && exp % 3 == 0 {
            Ok(Self { exp })
        } else {
            Err(InvalidArgument("Invalid exponent"))
        }
    }
}

/// Right y axis format.
///
/// See [`GraphProps`]
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct RightYAxis {
    pub scale: f64,
    pub shift: u32,
    pub label: Option<String>,
    pub formatter: Option<YAxisFormatter>,
    pub format: Option<String>,
}

impl AppendArgs for RightYAxis {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        args.push("--right-axis".to_string());
        args.push(format!("{}:{}", self.scale, self.shift));

        if let Some(l) = &self.label {
            args.push("--right-axis-label".to_string());
            args.push(l.clone());
        }

        if let Some(f) = &self.formatter {
            args.push("--right-axis-formatter".to_string());
            f.append_to(args)?;
        }

        if let Some(f) = &self.format {
            args.push("--right-axis-format".to_string());
            args.push(f.clone());
        }

        Ok(())
    }
}

/// Legend position and format.
///
/// See [`GraphProps`]
#[derive(Default, Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct Legend {
    pub no_legend: bool,
    pub force_rules_legend: bool,
    pub legend_position: Option<LegendPosition>,
    pub legend_direction: Option<LegendDirection>,
}

impl AppendArgs for Legend {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        if self.no_legend {
            args.push("--no-legend".to_string());
        }

        if self.force_rules_legend {
            args.push("--force-rules-legend".to_string());
        }

        if let Some(p) = self.legend_position {
            let pos = match p {
                LegendPosition::North => "north",
                LegendPosition::South => "south",
                LegendPosition::East => "east",
                LegendPosition::West => "west",
            }
            .to_string();
            args.push(format!("--legend-position={pos}"));
        }

        if let Some(d) = self.legend_direction {
            let dir = match d {
                LegendDirection::TopDown => "topdown",
                LegendDirection::BottomUp => "bottomup",
                LegendDirection::BottomUp2 => "bottomup2",
            }
            .to_string();
            args.push(format!("--legend-direction={dir}"));
        }

        Ok(())
    }
}

/// Legend position.
///
/// See [`Legend`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum LegendPosition {
    North,
    South,
    East,
    West,
}

/// Legend direction.
///
/// See [`Legend`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum LegendDirection {
    TopDown,
    BottomUp,
    BottomUp2,
}

/// Miscellaneous other properties.
///
/// See [`GraphProps`]
#[derive(Default, Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct Misc {
    // Skipping `lazy` as it is inapplicable when generating an in-memory graph
    // Skipping daemon as we don't support daemons
    // Skipping imginfo as the example usage is an antipattern (no html attr escaping),
    // and seeems better done in Rust logic anyway.
    pub colors: collections::HashMap<ColorTag, Color>,
    /// (on, off)
    pub grid_dash: Option<(u32, u32)>,
    pub border: Option<u32>,
    pub dynamic_labels: bool,
    pub zoom: Option<Zoom>,
    pub fonts: collections::HashMap<FontTag, FontParams>,
    pub font_render_mode: Option<FontRenderMode>,
    pub font_smoothing_threshold: Option<u32>,
    pub pango_markup: bool,
    pub graph_render_mode: Option<GraphRenderMode>,
    pub slope_mode: bool,
    // image_format is required, so it's a top-level fn param
    pub interlaced: bool,
    pub tab_width: Option<u32>,
    pub base: Option<u32>,
    pub watermark: Option<String>,
    pub use_nan_for_all_missing_data: bool,
}

impl AppendArgs for Misc {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        for (tag, color) in &self.colors {
            args.push("--color".to_string());
            let mut s = match tag {
                ColorTag::Back => "BACK",
                ColorTag::Canvas => "CANVAS",
                ColorTag::ShadeA => "SHADEA",
                ColorTag::ShadeB => "SHADEB",
                ColorTag::Grid => "GRID",
                ColorTag::MGrid => "MGRID",
                ColorTag::Font => "FONT",
                ColorTag::Axis => "AXIS",
                ColorTag::Frame => "FRAME",
                ColorTag::Arrow => "ARROW",
            }
            .to_string();
            color.append_to(&mut s);
            args.push(s);
        }

        if let Some((on, off)) = self.grid_dash {
            args.push("--grid-dash".to_string());
            args.push(format!("{}:{}", on, off));
        }

        if let Some(border) = self.border {
            args.push("--border".to_string());
            args.push(format!("{border}"));
        }

        if self.dynamic_labels {
            args.push("--dynamic-labels".to_string());
        }

        if let Some(z) = self.zoom {
            args.push("--zoom".to_string());
            args.push(format!("{}", z.zoom));
        }

        for (tag, font_params) in &self.fonts {
            args.push("--font".to_string());
            let tag = match tag {
                FontTag::Default => "DEFAULT",
                FontTag::Title => "TITLE",
                FontTag::Axis => "AXIS",
                FontTag::Unit => "UNIT",
                FontTag::Legend => "LEGEND",
                FontTag::Watermark => "WATERMARK",
            }
            .to_string();
            args.push(match &font_params.font {
                None => format!("{tag}:{}", font_params.size),
                Some(f) => format!("{tag}:{}:{f}", font_params.size),
            })
        }

        if let Some(frm) = &self.font_render_mode {
            frm.append_to(args)?;
        }

        if let Some(fst) = self.font_smoothing_threshold {
            args.push("--font-smoothing-threshold".to_string());
            args.push(format!("{fst}"));
        }

        if self.pango_markup {
            args.push("--pango-markup".to_string());
        }

        if let Some(grm) = &self.graph_render_mode {
            grm.append_to(args)?;
        }

        if self.slope_mode {
            args.push("--slope-mode".to_string());
        }

        if self.interlaced {
            args.push("--interlaced".to_string());
        }

        if let Some(t) = self.tab_width {
            args.push("--tabwidth".to_string());
            args.push(format!("{t}"));
        }

        if let Some(b) = self.base {
            args.push("--base".to_string());
            args.push(format!("{b}"));
        }

        if let Some(w) = &self.watermark {
            args.push("--watermark".to_string());
            args.push(w.clone());
        }

        if self.use_nan_for_all_missing_data {
            args.push("--use-nan-for-all-missing-data".to_string());
        }

        Ok(())
    }
}

/// Aspects that can have a color set.
///
/// See [`Misc`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum ColorTag {
    Back,
    Canvas,
    ShadeA,
    ShadeB,
    Grid,
    MGrid,
    Font,
    Axis,
    Frame,
    Arrow,
}

/// Zoom level.
///
/// See [`Misc`]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Zoom {
    zoom: f64,
}

impl Zoom {
    /// Returns `Some` if zoom > 0.
    pub fn new(zoom: f64) -> Result<Self, InvalidArgument> {
        if zoom > 0.0 {
            Ok(Self { zoom })
        } else {
            Err(InvalidArgument("zoom must be positive"))
        }
    }
}

/// Font size and name.
///
/// See [`Misc`]
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct FontParams {
    pub size: u32,
    pub font: Option<String>,
}

/// Text type that can have a font set.
///
/// See [`Misc`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum FontTag {
    Default,
    Title,
    Axis,
    Unit,
    Legend,
    Watermark,
}

/// Font render mode.
///
/// See [`Misc`]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum FontRenderMode {
    Normal,
    Light,
    Mono,
}

impl AppendArgs for FontRenderMode {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        args.push("--font-render-mode".to_string());
        args.push(
            match self {
                FontRenderMode::Normal => "normal",
                FontRenderMode::Light => "light",
                FontRenderMode::Mono => "mono",
            }
            .to_string(),
        );

        Ok(())
    }
}

/// Graph anti-aliasing render mode.
///
/// See [`Misc`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum GraphRenderMode {
    Normal,
    Mono,
}

impl AppendArgs for GraphRenderMode {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        args.push("--graph-render-mode".to_string());
        args.push(
            match self {
                GraphRenderMode::Normal => "normal",
                GraphRenderMode::Mono => "mono",
            }
            .to_string(),
        );

        Ok(())
    }
}

/// Image format to render the graph as.
///
/// See [`Misc`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum ImageFormat {
    Png,
    Svg,
    Eps,
    Pdf,
    // skipping non-image export formats
}

impl AppendArgs for ImageFormat {
    fn append_to(&self, args: &mut Vec<String>) -> RrdResult<()> {
        args.push("--imgformat".to_string());
        args.push(
            match self {
                ImageFormat::Png => "PNG",
                ImageFormat::Svg => "SVG",
                ImageFormat::Eps => "EPS",
                ImageFormat::Pdf => "PDF",
            }
            .to_string(),
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    // at least a baseline check that some sane args are produced
    #[test]
    fn everything_set() {
        let props = GraphProps {
            time_range: TimeRange {
                start: Some(Timestamp::from_time_t(1_000)),
                end: Some(Timestamp::from_time_t(100_000)),
                step_seconds: Some(60),
            },
            labels: Labels {
                title: Some("Title".to_string()),
                vertical_label: Some("VLabel".to_string()),
            },
            size: Size {
                width: Some(1024),
                height: Some(768),
                full_size_mode: true,
                only_graph: true,
            },
            limits: Limits {
                upper_limit: Some(100.0),
                lower_limit: Some(1.0),
                rigid: true,
                allow_shrink: true,
                alt_autoscale: Some(AltAutoscale {
                    alt_autoscale_min: Some(1.1),
                    alt_autoscale_max: Some(2.2),
                }),
                no_grid_fit: true,
            },
            x_axis: XAxis {
                grid: Some(XAxisGrid::Custom {
                    base_grid_time: AxisGridTimeUnit::Second,
                    base_grid_step: 1,
                    major_grid_time: AxisGridTimeUnit::Hour,
                    major_grid_step: 2,
                    labels_time: AxisGridTimeUnit::Month,
                    labels_step: 3,
                    label_placement: 4,
                    label_format: "label fmt".to_string(),
                }),
                week_format: Some("weekfmt".to_string()),
            },
            y_axis: YAxis {
                grid: Some(YAxisGrid::Custom {
                    grid_step: 100,
                    label_factor: 2,
                }),
                formatter: Some(YAxisFormatter::Numeric),
                format: Some("yaxisfmt".to_string()),
                alt_y_grid: true,
                logarithmic: true,
                units_exponent: Some(UnitsExponent { exp: 3 }),
                units_length: Some(4),
                units: Some(Units::Si),
            },
            right_y_axis: Some(RightYAxis {
                scale: 0.0,
                shift: 0,
                label: Some("right y axis label".to_string()),
                formatter: Some(YAxisFormatter::Numeric),
                format: Some("right y axis fmt".to_string()),
            }),
            legend: Legend {
                no_legend: true,
                force_rules_legend: true,
                legend_position: Some(LegendPosition::North),
                legend_direction: Some(LegendDirection::BottomUp),
            },
            misc: Misc {
                colors: [(ColorTag::Axis, "#01020304".parse().unwrap())]
                    .into_iter()
                    .collect(),
                grid_dash: Some((1, 2)),
                border: Some(4),
                dynamic_labels: true,
                zoom: Some(Zoom { zoom: 3.3 }),
                fonts: [(
                    FontTag::Unit,
                    FontParams {
                        size: 11,
                        font: Some("FontyMcFontFace".to_string()),
                    },
                )]
                .into_iter()
                .collect(),
                font_render_mode: Some(FontRenderMode::Mono),
                font_smoothing_threshold: Some(1234),
                pango_markup: true,
                graph_render_mode: Some(GraphRenderMode::Mono),
                slope_mode: true,
                interlaced: true,
                tab_width: Some(7),
                base: Some(4),
                watermark: Some("watermark".to_string()),
                use_nan_for_all_missing_data: true,
            },
        };

        let mut args = vec![];
        props.append_to(&mut args).unwrap();

        let expected = [
            // time range
            "--start",
            "1000",
            "--end",
            "100000",
            "--step",
            "60",
            // labels
            "--title",
            "Title",
            "--vertical-label",
            "VLabel",
            // size
            "--width",
            "1024",
            "--height",
            "768",
            "--only-graph",
            "--full-size-mode",
            // limits
            "--upper-limit",
            "100",
            "--lower-limit",
            "1",
            "--rigid",
            "--allow-shrink",
            "--alt-autoscale",
            "--alt-autoscale-min",
            "1.1",
            "--alt-autoscale-max",
            "2.2",
            "--no-gridfit",
            // x axis
            "--x-grid",
            "SECOND:1:HOUR:2:MONTH:3:4:label fmt",
            "--week-fmt",
            "weekfmt",
            // y axis
            "--y-grid",
            "100:2",
            "--left-axis-formatter",
            "numeric",
            "--left-axis-format",
            "yaxisfmt",
            "--alt-y-grid",
            "--logarithmic",
            "--units-exponent",
            "3",
            "--units-length",
            "4",
            "--units",
            "si",
            // right y axis
            "--right-axis",
            "0:0",
            "--right-axis-label",
            "right y axis label",
            "--right-axis-formatter",
            "numeric",
            "--right-axis-format",
            "right y axis fmt",
            // legend
            "--no-legend",
            "--force-rules-legend",
            "--legend-position=north",
            "--legend-direction=bottomup",
            // misc
            "--color",
            "AXIS#01020304",
            "--grid-dash",
            "1:2",
            "--border",
            "4",
            "--dynamic-labels",
            "--zoom",
            "3.3",
            "--font",
            "UNIT:11:FontyMcFontFace",
            "--font-render-mode",
            "mono",
            "--font-smoothing-threshold",
            "1234",
            "--pango-markup",
            "--graph-render-mode",
            "mono",
            "--slope-mode",
            "--interlaced",
            "--tabwidth",
            "7",
            "--base",
            "4",
            "--watermark",
            "watermark",
            "--use-nan-for-all-missing-data",
        ];

        assert_eq!(
            expected.into_iter().map(|s| s.to_string()).collect_vec(),
            args
        );
    }
}
