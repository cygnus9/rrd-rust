use rrd::{ops::create, ConsolidationFn};
use std::{path::Path, time::Duration};

fn main() {
    let filename = Path::new("db.rrd");

    create::create(
        filename,
        Duration::from_secs(1),
        chrono::Utc::now(),
        false,
        &[],
        None,
        &[
            create::DataSource::gauge(
                create::DataSourceName::new("volt"),
                300,
                Some(0.0),
                Some(24000.0),
            ),
            create::DataSource::gauge(
                create::DataSourceName::new("amps"),
                300,
                Some(0.0),
                Some(24000.0),
            ),
            create::DataSource::compute(create::DataSourceName::new("watts"), "volt,amps,*"),
        ],
        &[create::Archive::new(ConsolidationFn::Avg, 0.5, 1, 86400).unwrap()],
    )
    .expect("Failed to create db");

    let rc = rrd::update(filename, None, rrd::ExtraFlags::empty(), &["N:235:12.3"]);
    match rc {
        Ok(_) => println!("Ok"),
        Err(err) => println!("Not ok: {err}"),
    }
}
