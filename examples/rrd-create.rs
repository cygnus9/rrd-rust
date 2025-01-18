use rrd::{ops::create, ConsolidationFn};
use std::{path::Path, time::Duration};

fn main() {
    let rc = create::create(
        Path::new("db.rrd"),
        chrono::Utc::now(),
        Duration::from_secs(1),
        false,
        None,
        &[],
        &[create::DataSource::gauge(
            create::DataSourceName::new("watts"),
            300,
            Some(0.0),
            Some(24000.0),
        )],
        &[create::Archive::new(ConsolidationFn::Avg, 0.5, 1, 86400).unwrap()],
    );
    match rc {
        Ok(_) => println!("Ok"),
        Err(err) => println!("Not ok: {err}"),
    }
}
