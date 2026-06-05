use rrd::{ops::create, ConsolidationFn};
use std::{
    path::Path,
    time::{Duration, SystemTime},
};

fn main() {
    let rc = create::create(
        Path::new("db.rrd"),
        SystemTime::now(),
        Duration::from_secs(1),
        false,
        None,
        &[],
        &[create::DataSource::gauge(
            &create::DataSourceName::new("watts").unwrap(),
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
