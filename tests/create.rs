use itertools::Itertools;
use rrd::{ops::create, ops::info, ConsolidationFn};
use std::{collections, time};

#[test]
fn create_all_ds_types() -> anyhow::Result<()> {
    let tempdir = tempfile::tempdir()?;
    let rrd_path = tempdir.path().join("rrd");
    let now = chrono::Utc::now();
    create::create(
        &rrd_path,
        now,
        time::Duration::from_secs(1),
        true,
        None,
        &[],
        &[
            create::DataSource::gauge(
                create::DataSourceName::new("gauge"),
                300,
                Some(0.0),
                Some(1000.0),
            ),
            create::DataSource::counter(
                create::DataSourceName::new("counter"),
                300,
                Some(0),
                Some(1000),
            ),
            create::DataSource::dcounter(
                create::DataSourceName::new("dcounter"),
                300,
                Some(0.0),
                Some(1000.0),
            ),
            create::DataSource::derive(
                create::DataSourceName::new("derive"),
                300,
                Some(0),
                Some(1000),
            ),
            create::DataSource::dderive(
                create::DataSourceName::new("dderive"),
                300,
                Some(0.0),
                Some(1000.0),
            ),
            create::DataSource::absolute(
                create::DataSourceName::new("absolute"),
                300,
                Some(0),
                Some(1000),
            ),
            create::DataSource::compute(create::DataSourceName::new("compute"), "gauge,counter,+"),
        ],
        &[create::Archive::new(ConsolidationFn::Avg, 0.5, 6, 10).unwrap()],
    )?;

    let mut info = info::info(&rrd_path)?;
    // these keys vary every time
    for k in [
        "rra[0].cdp_prep[0].unknown_datapoints",
        "rra[0].cdp_prep[1].unknown_datapoints",
        "rra[0].cdp_prep[2].unknown_datapoints",
        "rra[0].cdp_prep[3].unknown_datapoints",
        "rra[0].cdp_prep[4].unknown_datapoints",
        "rra[0].cdp_prep[5].unknown_datapoints",
        "rra[0].cdp_prep[6].unknown_datapoints",
        "rra[0].cur_row",
    ] {
        assert!(info.remove(k).is_some());
    }

    let expected: collections::HashMap<String, info::InfoValue> = [
        ("ds[absolute].index", 5_u64.into()),
        ("ds[absolute].last_ds", "U".into()),
        ("ds[absolute].max", 1000.00_f64.into()),
        ("ds[absolute].min", 0.00_f64.into()),
        ("ds[absolute].minimal_heartbeat", 300_u64.into()),
        ("ds[absolute].type", "ABSOLUTE".into()),
        ("ds[absolute].unknown_sec", 0_u64.into()),
        ("ds[absolute].value", f64::NAN.into()),
        ("ds[compute].cdef", "gauge,counter,+".into()),
        ("ds[compute].index", 6_u64.into()),
        ("ds[compute].last_ds", "U".into()),
        ("ds[compute].type", "COMPUTE".into()),
        ("ds[compute].unknown_sec", 0_u64.into()),
        ("ds[compute].value", f64::NAN.into()),
        ("ds[counter].index", 1_u64.into()),
        ("ds[counter].last_ds", "U".into()),
        ("ds[counter].max", 1000.00_f64.into()),
        ("ds[counter].min", 0.00_f64.into()),
        ("ds[counter].minimal_heartbeat", 300_u64.into()),
        ("ds[counter].type", "COUNTER".into()),
        ("ds[counter].unknown_sec", 0_u64.into()),
        ("ds[counter].value", f64::NAN.into()),
        ("ds[dcounter].index", 2_u64.into()),
        ("ds[dcounter].last_ds", "U".into()),
        ("ds[dcounter].max", 1000.00_f64.into()),
        ("ds[dcounter].min", 0.00_f64.into()),
        ("ds[dcounter].minimal_heartbeat", 300_u64.into()),
        ("ds[dcounter].type", "DCOUNTER".into()),
        ("ds[dcounter].unknown_sec", 0_u64.into()),
        ("ds[dcounter].value", f64::NAN.into()),
        ("ds[dderive].index", 4_u64.into()),
        ("ds[dderive].last_ds", "U".into()),
        ("ds[dderive].max", 1000.00_f64.into()),
        ("ds[dderive].min", 0.00_f64.into()),
        ("ds[dderive].minimal_heartbeat", 300_u64.into()),
        ("ds[dderive].type", "DDERIVE".into()),
        ("ds[dderive].unknown_sec", 0_u64.into()),
        ("ds[dderive].value", f64::NAN.into()),
        ("ds[derive].index", 3_u64.into()),
        ("ds[derive].last_ds", "U".into()),
        ("ds[derive].max", 1000.00_f64.into()),
        ("ds[derive].min", 0.00_f64.into()),
        ("ds[derive].minimal_heartbeat", 300_u64.into()),
        ("ds[derive].type", "DERIVE".into()),
        ("ds[derive].unknown_sec", 0_u64.into()),
        ("ds[derive].value", f64::NAN.into()),
        ("ds[gauge].index", 0_u64.into()),
        ("ds[gauge].last_ds", "U".into()),
        ("ds[gauge].max", 1000.00_f64.into()),
        ("ds[gauge].min", 0.00_f64.into()),
        ("ds[gauge].minimal_heartbeat", 300_u64.into()),
        ("ds[gauge].type", "GAUGE".into()),
        ("ds[gauge].unknown_sec", 0_u64.into()),
        ("ds[gauge].value", f64::NAN.into()),
        ("header_size", 2456_u64.into()),
        ("rra[0].cdp_prep[0].value", f64::NAN.into()),
        ("rra[0].cdp_prep[1].value", f64::NAN.into()),
        ("rra[0].cdp_prep[2].value", f64::NAN.into()),
        ("rra[0].cdp_prep[3].value", f64::NAN.into()),
        ("rra[0].cdp_prep[4].value", f64::NAN.into()),
        ("rra[0].cdp_prep[5].value", f64::NAN.into()),
        ("rra[0].cdp_prep[6].value", f64::NAN.into()),
        ("rra[0].cf", "AVERAGE".into()),
        ("rra[0].pdp_per_row", 6_u64.into()),
        ("rra[0].rows", 10_u64.into()),
        ("rra[0].xff", 0.50_f64.into()),
        ("rrd_version", "0005".into()),
        ("step", 1_u64.into()),
        ("last_update", u64::try_from(now.timestamp())?.into()),
        ("filename", rrd_path.to_string_lossy().into_owned().into()),
    ]
    .into_iter()
    .map(|(k, v): (&str, info::InfoValue)| (k.to_string(), v))
    .collect();

    // sort by key only to avoid sorting floats, and exclude NaNs
    let expected_vec = expected
        .into_iter()
        .sorted_by_key(|(key, _)| key.clone())
        .collect_vec();
    let actual_vec = info
        .into_iter()
        .sorted_by_key(|(key, _)| key.clone())
        .collect_vec();
    assert_eq!(
        expected_vec
            .iter()
            .filter(|(_k, v)| !is_nan_float(v))
            .collect_vec(),
        actual_vec
            .iter()
            .filter(|(_k, v)| !is_nan_float(v))
            .collect_vec()
    );

    // make sure the NaNs are the same
    assert_eq!(
        expected_vec
            .iter()
            .filter(|(_k, v)| is_nan_float(v))
            .map(|(k, _v)| k)
            .collect_vec(),
        actual_vec
            .iter()
            .filter(|(_k, v)| is_nan_float(v))
            .map(|(k, _v)| k)
            .collect_vec(),
    );

    Ok(())
}

fn is_nan_float(v: &info::InfoValue) -> bool {
    match v {
        info::InfoValue::Value(f) => f.is_nan(),
        _ => false,
    }
}
