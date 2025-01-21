use itertools::Itertools;
use rrd::{
    ops::{
        create, fetch, graph,
        graph::{commands, props},
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
        update::update_all(&rrd_path, update::ExtraFlags::empty(), chunk)?;
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
        fetched.sources().iter().map(|ds| ds.name()).collect_vec()
    );

    let fetched_timestamps = fetched.rows().iter().map(|r| r.timestamp()).collect_vec();

    let expected = [
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

    assert_eq!(
        expected.iter().map(|(ts, _val)| *ts).collect_vec(),
        fetched_timestamps
    );

    let graph_start = Timestamp::from_timestamp(920804400, 0).unwrap();
    let graph_end = Timestamp::from_timestamp(920808000, 0).unwrap();

    let initial_expected_metadata = graph::GraphMetadata {
        graph_left: 51,
        graph_top: 15,
        graph_width: 400,
        graph_height: 100,
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
        let var_name: commands::VarName = "myspeed".try_into()?;
        let (png_data, metadata) = graph::graph(
            props::ImageFormat::Png,
            props::GraphProps {
                time_range: props::TimeRange {
                    start: Some(graph_start),
                    end: Some(graph_end),
                    ..Default::default()
                },
                ..Default::default()
            },
            &[
                commands::Def {
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
                commands::Line {
                    width: 2.0,
                    value: var_name.into(),
                    color: Some(commands::ColorWithLegend {
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
        let myspeed: commands::VarName = "myspeed".try_into()?;
        let realspeed = "realspeed".try_into()?;
        let (png_data, metadata) = graph::graph(
            props::ImageFormat::Png,
            props::GraphProps {
                time_range: props::TimeRange {
                    start: Some(graph_start),
                    end: Some(graph_end),
                    ..Default::default()
                },
                ..Default::default()
            },
            &[
                commands::Def {
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
                commands::CDef {
                    var_name: realspeed,
                    rpn: "myspeed,1000,*".to_string(),
                }
                .into(),
                commands::Line {
                    width: 2.0,
                    value: myspeed.into(),
                    color: Some(commands::ColorWithLegend {
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
        let myspeed: commands::VarName = "myspeed".try_into()?;
        let good: commands::VarName = "good".try_into()?;
        let fast: commands::VarName = "fast".try_into()?;
        let (png_data, metadata) = graph::graph(
            props::ImageFormat::Png,
            props::GraphProps {
                time_range: props::TimeRange {
                    start: Some(graph_start),
                    end: Some(graph_end),
                    ..Default::default()
                },
                labels: props::Labels {
                    vertical_label: Some("km/h".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            &[
                commands::Def {
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
                commands::CDef {
                    var_name: "kmh".try_into()?,
                    rpn: "myspeed,3600,*".to_string(),
                }
                .into(),
                commands::CDef {
                    var_name: fast.clone(),
                    rpn: "kmh,100,GT,kmh,0,IF".to_string(),
                }
                .into(),
                commands::CDef {
                    var_name: good.clone(),
                    rpn: "kmh,100,GT,0,kmh,IF".to_string(),
                }
                .into(),
                commands::HRule {
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
                commands::Area {
                    value: good.into(),
                    color: Some(commands::ColorWithLegend {
                        color: commands::AreaColor::Color(graph::Color {
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
                commands::Area {
                    value: fast.into(),
                    color: Some(commands::ColorWithLegend {
                        color: commands::AreaColor::Color(graph::Color {
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
        let expected = graph::GraphMetadata {
            graph_left: 67,
            image_width: 497,
            image_height: 155,
            value_max: 200.0,
            extra_info: [
                ("legend[0]", "  Maximum allowed".into()),
                ("legend[1]", "  Good speed".into()),
                ("legend[2]", "  Too fast".into()),
                ("coords[0]", "16,134,135,148".into()),
                ("coords[1]", "231,134,315,148".into()),
                ("coords[2]", "411,134,481,148".into()),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
            ..initial_expected_metadata
        };

        assert_eq!(expected, metadata);
    }

    Ok(())
}
