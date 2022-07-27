use std::path::Path;
use std::time::{Duration, SystemTime};

use rrd::ExtraFlags;

fn main() {
    let filename = Path::new("db.rrd");

    rrd::create(
        filename,
        Duration::from_secs(1),
        &SystemTime::now(),
        false,
        &[],
        None,
        &[
            "DS:volt:GAUGE:300:0:24000",
            "DS:amps:GAUGE:300:0:24000",
            "RRA:AVERAGE:0.5:1:864000",
        ],
    )
    .expect("Failed to create db");

    let rc = rrd::update(filename, None, ExtraFlags::empty(), &["N:235:12.3"]);
    match rc {
        Ok(_) => println!("Ok"),
        Err(err) => println!("Not ok: {err}"),
    }
}
