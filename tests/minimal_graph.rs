use rrd::{
    error::RrdResult,
    ops::{
        create, graph,
        graph::{elements, props},
        update,
    },
    ConsolidationFn, Timestamp,
};
use std::{path, time};

#[test]
fn minimal_graph() -> anyhow::Result<()> {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    // shell sequence that works:
    // rrdcreate tmp/data.rrd --start 1737317206 --step 1 --no-overwrite DS:gauge:GAUGE:300:0:1000 RRA:AVERAGE:0.5:1:1000
    // rrdupdate tmp/data.rrd --template gauge 1737317211:10
    // rrdtool graph tmp/graph.png --imgformat PNG DEF:g=tmp/data.rrd:gauge:AVERAGE LINE4:g

    let tempdir = tempfile::tempdir()?;
    let rrd_path = tempdir.path().join("data.rrd");
    let ds_name = "gauge";
    create::create(
        &rrd_path,
        // must be before the update timestamp or update will silently fail
        Timestamp::from_timestamp(1737317206, 0).unwrap(),
        time::Duration::from_secs(1),
        true,
        None,
        &[],
        &[create::DataSource::gauge(
            create::DataSourceName::new(ds_name),
            300,
            Some(0.0),
            Some(1000.0),
        )],
        &[create::Archive::new(ConsolidationFn::Avg, 0.5, 1, 1000).unwrap()],
    )?;
    assert!(rrd_path.exists());

    let data_point_time = Timestamp::from_timestamp(1737317211, 0).unwrap();
    update::update(
        &rrd_path,
        &[ds_name],
        update::ExtraFlags::empty(),
        &[
            (data_point_time.into(), [10.into()]),
            (
                (data_point_time + time::Duration::from_secs(60)).into(),
                [10.into()],
            ),
        ],
    )?;

    // make sure all the formats work

    {
        let image = build_graph(rrd_path.clone(), props::ImageFormat::Png, ds_name)?;
        // png signature
        assert_eq!(b"\x89PNG\r\n\x1a\n", &image[..8]);
    }

    {
        let image = build_graph(rrd_path.clone(), props::ImageFormat::Svg, ds_name)?;
        assert!(
            image.starts_with(br#"<?xml version="1.0" encoding="UTF-8"?>"#),
            "{}",
            String::from_utf8_lossy(&image)
                .chars()
                .take(100)
                .collect::<String>()
        );
    }

    {
        let image = build_graph(rrd_path.clone(), props::ImageFormat::Eps, ds_name)?;
        assert!(
            image.starts_with(br#"%!PS-Adobe-3.0"#),
            "{}",
            String::from_utf8_lossy(&image)
                .chars()
                .take(100)
                .collect::<String>()
        );
    }

    {
        let image = build_graph(rrd_path.clone(), props::ImageFormat::Pdf, ds_name)?;
        // PDF version varies based on system dependencies -- presumably pango or something
        assert!(
            image.starts_with(br#"%PDF-1."#),
            "{}",
            String::from_utf8_lossy(&image)
                .chars()
                .take(100)
                .collect::<String>()
        );
    }

    Ok(())
}

fn build_graph(
    rrd_path: path::PathBuf,
    img_format: props::ImageFormat,
    ds_name: &str,
) -> RrdResult<Vec<u8>> {
    let var_name_g = elements::VarName::new("g".to_string())?;
    // a little before and a little after the data points in update()
    let start = Timestamp::from_timestamp(1737316000, 0).unwrap();
    let end = Timestamp::from_timestamp(1737319000, 0).unwrap();

    let (image, metadata) = graph::graph(
        img_format,
        props::GraphProps {
            time_range: props::TimeRange {
                start: Some(start),
                end: Some(end),
                ..Default::default()
            },
            ..Default::default()
        },
        &[
            elements::Def {
                var_name: var_name_g.clone(),
                rrd: rrd_path,
                ds_name: ds_name.to_string(),
                consolidation_fn: ConsolidationFn::Avg,
                step: None,
                start: None,
                end: None,
                reduce: None,
            }
            .into(),
            elements::Line {
                width: 4.0,
                value: var_name_g.clone(),
                color: None,
                stack: false,
                skip_scale: false,
                dashes: None,
            }
            .into(),
        ],
    )?;

    assert_eq!(
        graph::GraphMetadata {
            graph_left: 51,
            graph_top: 15,
            graph_width: 400,
            graph_height: 100,
            graph_start: start,
            graph_end: end,
            image_width: 481,
            image_height: 141,
            // not clear why these are 9 and 20, but none of this is documented so...
            value_min: 9.0,
            value_max: 20.0,
            extra_info: Default::default(),
        },
        metadata
    );

    Ok(image)
}
