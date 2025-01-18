use std::{f64::consts::PI, path::Path, time::Duration};

use rrd::{ops::create, ConsolidationFn};

fn main() {
    let filename = Path::new("db.rrd");
    let start = chrono::Utc::now();
    let end = start + chrono::TimeDelta::seconds(300);

    create::create(
        filename,
        Duration::from_secs(1),
        start - chrono::TimeDelta::seconds(1),
        false,
        &[],
        None,
        &[
            create::DataSource::gauge(
                create::DataSourceName::new("sin"),
                10,
                Some(-1.0),
                Some(1.0),
            ),
            create::DataSource::gauge(
                create::DataSourceName::new("cos"),
                10,
                Some(-1.0),
                Some(1.0),
            ),
        ],
        &[
            create::Archive::new(ConsolidationFn::Avg, 0.5, 1, 300).unwrap(),
            create::Archive::new(ConsolidationFn::Avg, 0.5, 5, 300).unwrap(),
        ],
    )
    .expect("Failed to create db");

    for offset in 0..300 {
        let ts = start + Duration::from_secs(offset);
        let x = offset as f64 * PI / 300f64;
        let sin_value = x.sin();
        let cos_value = x.cos();

        let s = format!("{}:{sin_value:.3}:{cos_value:.3}", ts.timestamp());
        rrd::update(filename, None, rrd::ExtraFlags::empty(), &[&s]).unwrap();
    }

    let rc = rrd::fetch(filename, "AVERAGE", start, end, Duration::from_secs(1));
    match rc {
        Ok(data) => {
            println!("Ok");
            println!("  Start: {}", data.start());
            println!("  End: {}", data.end());
            println!("  Step: {:?}", data.step());
            println!("  Rows: {}", data.row_count());

            let sources = data.sources();
            println!("  Data sources: {}", sources.len());
            for (i, source) in sources.iter().enumerate() {
                println!("    #{}: {}", i, source.name());
            }
            let rows = data.rows();
            println!("  Rows: {}", rows.len());
            for (i, row) in rows.iter().enumerate() {
                println!(
                    "    #{:03}: {} - {:.03}, {:.03}",
                    i,
                    row.timestamp(),
                    row[0].value,
                    row[1].value
                );
            }
        }
        Err(err) => println!("Not ok: {err}"),
    }
}
