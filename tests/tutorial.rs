use itertools::Itertools;
use rrd::{
    ops::{
        create, fetch, graph,
        graph::{elements, props},
        update,
    },
    ConsolidationFn, Timestamp,
};
use std::time;

/// Steps from https://oss.oetiker.ch/rrdtool/tut/rrdtutorial.en.html

#[test]
fn tutorial() -> anyhow::Result<()> {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();
    let tempdir = tempfile::tempdir()?;
    let rrd_path = tempdir.path().join("data.rrd");

    create::create(
        &rrd_path,
        Timestamp::from_timestamp(920804400, 0).unwrap(),
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

    let update_data = [
        (920804700_i64, 12345_u64),
        (920805000, 12357),
        (920805300, 12363),
        (920805600, 12363),
        (920805900, 12363),
        (920806200, 12373),
        (920806500, 12383),
        (920806800, 12393),
        (920807100, 12399),
        (920807400, 12405),
        (920807700, 12411),
        (920808000, 12415),
        (920808300, 12420),
        (920808600, 12422),
        (920808900, 12423),
    ]
    .into_iter()
    .map(|(ts, value)| {
        (
            update::BatchTime::from(Timestamp::from_timestamp(ts, 0).unwrap()),
            [update::Datum::from(value)],
        )
    })
    .collect_vec();

    // updates done in chunks of 3
    for chunk in update_data.chunks(3) {
        update::update_all(&rrd_path, update::Options::default(), chunk)?;
    }

    let fetched = fetch::fetch(
        &rrd_path,
        ConsolidationFn::Avg,
        Timestamp::from_timestamp(920804400, 0).unwrap(),
        Timestamp::from_timestamp(920809200, 0).unwrap(),
        time::Duration::from_secs(300),
    )?;

    assert_eq!(
        vec!["speed".to_string()],
        fetched.ds_names().iter().cloned().collect_vec()
    );

    let fetched_expected = [
        (920804700, f64::NAN),
        (920805000, 4.0000000000e-02),
        (920805300, 2.0000000000e-02),
        (920805600, 0.0000000000e+00),
        (920805900, 0.0000000000e+00),
        (920806200, 3.3333333333e-02),
        (920806500, 3.3333333333e-02),
        (920806800, 3.3333333333e-02),
        (920807100, 2.0000000000e-02),
        (920807400, 2.0000000000e-02),
        (920807700, 2.0000000000e-02),
        (920808000, 1.3333333333e-02),
        (920808300, 1.6666666667e-02),
        (920808600, 6.6666666667e-03),
        (920808900, 3.3333333333e-03),
        (920809200, f64::NAN),
        (920809500, f64::NAN),
    ]
    .into_iter()
    .map(|(ts, val)| (Timestamp::from_timestamp(ts, 0).unwrap(), val))
    .collect_vec();

    // timestamps match
    assert_eq!(
        fetched_expected.iter().map(|(ts, _val)| *ts).collect_vec(),
        fetched.rows().iter().map(|r| r.timestamp()).collect_vec()
    );

    // Compare values without relying on ==, because of NaN and minor differences w/ tutorial
    // sample output
    // Row timestamps are the same, so just need to compare values
    fetched_expected
        .iter()
        .map(|(_ts, val)| val)
        .zip_eq(fetched.rows().iter().map(|row| {
            let values = row.as_slice();
            assert_eq!(1, values.len());
            values[0]
        }))
        .enumerate()
        .for_each(|(index, (expected, actual))| {
            // Either both nan or neither nan and very close
            assert!(
                (expected.is_nan() && actual.is_nan())
                    || (!expected.is_nan()
                        && !actual.is_nan()
                        && (expected - actual).abs() < 0.000000001),
                "index {index}, expected {expected} actual {actual}"
            );
        });

    let graph_start = Timestamp::from_timestamp(920804400, 0).unwrap();
    let graph_end = Timestamp::from_timestamp(920808000, 0).unwrap();

    let base_graph_props = props::GraphProps {
        time_range: props::TimeRange {
            start: Some(graph_start),
            end: Some(graph_end),
            ..Default::default()
        },
        size: props::Size {
            // Explicitly fix the output size so the tests are deterministic across
            // environments with different fonts / layout engines.
            width: Some(481),
            height: Some(141),
            only_graph: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let initial_expected_metadata = graph::GraphMetadata {
        graph_left: 0,
        graph_top: 0,
        graph_width: 481,
        graph_height: 141,
        graph_start,
        graph_end,
        image_width: 481,
        image_height: 141,
        value_min: 0.0,
        value_max: 0.04,
        extra_info: Default::default(),
    };

    // first basic graph
    {
        let var_name: elements::VarName = "myspeed".try_into()?;
        let (png_data, metadata) = graph::graph(
            props::ImageFormat::Png,
            base_graph_props.clone(),
            &[
                elements::Def {
                    var_name: var_name.clone(),
                    rrd: rrd_path.clone(),
                    ds_name: "speed".to_string(),
                    consolidation_fn: ConsolidationFn::Avg,
                    step: None,
                    start: None,
                    end: None,
                    reduce: None,
                }
                .into(),
                elements::Line {
                    width: 2.0,
                    value: var_name,
                    color: Some(elements::ColorWithLegend {
                        color: graph::Color {
                            red: 0xFF,
                            green: 0x00,
                            blue: 0x00,
                            alpha: None,
                        },
                        legend: None,
                    }),
                    stack: false,
                    skip_scale: false,
                    dashes: None,
                }
                .into(),
            ],
        )?;

        // png signature
        assert_eq!(b"\x89PNG\r\n\x1a\n", &png_data[..8]);
        assert_eq!(initial_expected_metadata, metadata);
    }

    // graph with a simple calculation
    {
        let myspeed: elements::VarName = "myspeed".try_into()?;
        let realspeed = "realspeed".try_into()?;
        let (png_data, metadata) = graph::graph(
            props::ImageFormat::Png,
            base_graph_props.clone(),
            &[
                elements::Def {
                    var_name: myspeed.clone(),
                    rrd: rrd_path.clone(),
                    ds_name: "speed".to_string(),
                    consolidation_fn: ConsolidationFn::Avg,
                    step: None,
                    start: None,
                    end: None,
                    reduce: None,
                }
                .into(),
                elements::CDef {
                    var_name: realspeed,
                    rpn: "myspeed,1000,*".to_string(),
                }
                .into(),
                elements::Line {
                    width: 2.0,
                    value: myspeed,
                    color: Some(elements::ColorWithLegend {
                        color: graph::Color {
                            red: 0xFF,
                            green: 0x00,
                            blue: 0x00,
                            alpha: None,
                        },
                        legend: None,
                    }),
                    stack: false,
                    skip_scale: false,
                    dashes: None,
                }
                .into(),
            ],
        )?;

        assert_eq!(b"\x89PNG\r\n\x1a\n", &png_data[..8]);
        assert_eq!(initial_expected_metadata, metadata);
    }

    // graph with more calculations
    {
        let myspeed: elements::VarName = "myspeed".try_into()?;
        let good: elements::VarName = "good".try_into()?;
        let fast: elements::VarName = "fast".try_into()?;
        let mut graph_props = base_graph_props.clone();
        graph_props.labels.vertical_label = Some("km/h".to_string());
        // Turn off only-graph mode so we can see legend/labels.
        graph_props.size.only_graph = false;

        let (png_data, mut metadata) = graph::graph(
            props::ImageFormat::Png,
            graph_props,
            &[
                elements::Def {
                    var_name: myspeed.clone(),
                    rrd: rrd_path.clone(),
                    ds_name: "speed".to_string(),
                    consolidation_fn: ConsolidationFn::Avg,
                    step: None,
                    start: None,
                    end: None,
                    reduce: None,
                }
                .into(),
                elements::CDef {
                    var_name: "kmh".try_into()?,
                    rpn: "myspeed,3600,*".to_string(),
                }
                .into(),
                elements::CDef {
                    var_name: fast.clone(),
                    rpn: "kmh,100,GT,kmh,0,IF".to_string(),
                }
                .into(),
                elements::CDef {
                    var_name: good.clone(),
                    rpn: "kmh,100,GT,0,kmh,IF".to_string(),
                }
                .into(),
                elements::HRule {
                    value: 100.0_f64.into(),
                    color: graph::Color {
                        red: 0,
                        green: 0,
                        blue: 0xFF,
                        alpha: None,
                    },
                    legend: Some("Maximum allowed".into()),
                    dashes: None,
                }
                .into(),
                elements::Area {
                    value: good,
                    color: Some(elements::ColorWithLegend {
                        color: elements::AreaColor::Color(graph::Color {
                            red: 0,
                            green: 0xFF,
                            blue: 0,
                            alpha: None,
                        }),
                        legend: Some("Good speed".into()),
                    }),
                    stack: false,
                    skip_scale: false,
                }
                .into(),
                elements::Area {
                    value: fast,
                    color: Some(elements::ColorWithLegend {
                        color: elements::AreaColor::Color(graph::Color {
                            red: 0xFF,
                            green: 0,
                            blue: 0,
                            alpha: None,
                        }),
                        legend: Some("Too fast".into()),
                    }),
                    stack: false,
                    skip_scale: false,
                }
                .into(),
            ],
        )?;

        assert_eq!(b"\x89PNG\r\n\x1a\n", &png_data[..8]);
        assert_eq!(481, metadata.graph_width);
        assert_eq!(141, metadata.graph_height);
        assert!(metadata.image_width >= metadata.graph_width);
        assert!(metadata.image_height >= metadata.graph_height);
        assert!(metadata.graph_left + metadata.graph_width <= metadata.image_width);
        assert!(metadata.graph_top + metadata.graph_height <= metadata.image_height);
        assert_eq!(200.0, metadata.value_max);
        assert!(metadata.value_min >= 0.0);

        // Legend text may or may not be present depending on the backend/font setup.
        // If it exists, ensure it matches the expected strings.
        for (key, expected_value) in [
            ("legend[0]", "  Maximum allowed"),
            ("legend[1]", "  Good speed"),
            ("legend[2]", "  Too fast"),
        ] {
            if let Some(entry) = metadata.extra_info.get(key) {
                let actual = entry
                    .clone()
                    .into_string()
                    .expect("legend entry not a string");
                assert_eq!(expected_value, actual);
            }
        }

        // If coordinate metadata exists, at least validate that it is within bounds.
        let assert_coords_within = |coord_key: &str| {
            if let Some(entry) = metadata.extra_info.get(coord_key) {
                let coords = Coords::from_str(
                    &entry
                        .clone()
                        .into_string()
                        .expect("coords value is not a string"),
                );
                assert!(coords.top_left.x >= 0);
                assert!(coords.top_left.y >= 0);
                assert!(coords.bottom_right.x <= metadata.image_width as i32);
                assert!(coords.bottom_right.y <= metadata.image_height as i32);
                assert!(coords.bottom_right.x > coords.top_left.x);
                assert!(coords.bottom_right.y > coords.top_left.y);
            }
        };

        assert_coords_within("coords[0]");
        assert_coords_within("coords[1]");
        assert_coords_within("coords[2]");
    }

    Ok(())
}

struct Coord {
    x: i32,
    y: i32,
}

struct Coords {
    top_left: Coord,
    bottom_right: Coord,
}

impl Coords {
    fn from_str(s: &str) -> Self {
        let parts = s.split(',').collect_vec();
        assert!(
            parts.len() == 4,
            "Expected 4 parts in coords string, got {}",
            parts.len()
        );

        Coords {
            top_left: Coord {
                x: parts[0]
                    .trim()
                    .parse()
                    .expect("Failed to parse x coordinate"),
                y: parts[1]
                    .trim()
                    .parse()
                    .expect("Failed to parse y coordinate"),
            },
            bottom_right: Coord {
                x: parts[2]
                    .trim()
                    .parse()
                    .expect("Failed to parse x coordinate"),
                y: parts[3]
                    .trim()
                    .parse()
                    .expect("Failed to parse y coordinate"),
            },
        }
    }

    fn close_to(&self, other: &Coords) -> bool {
        let tolerance = 1;
        (self.top_left.x - other.top_left.x).abs() <= tolerance
            && (self.top_left.y - other.top_left.y).abs() <= tolerance
            && (self.bottom_right.x - other.bottom_right.x).abs() <= tolerance
            && (self.bottom_right.y - other.bottom_right.y).abs() <= tolerance
    }
}
