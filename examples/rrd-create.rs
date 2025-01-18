use std::path::Path;
use std::time::Duration;

fn main() {
    let rc = rrd::create(
        Path::new("db.rrd"),
        Duration::from_secs(1),
        chrono::Utc::now(),
        false,
        &[],
        None,
        &["DS:watts:GAUGE:300:0:24000", "RRA:AVERAGE:0.5:1:864000"],
    );
    match rc {
        Ok(_) => println!("Ok"),
        Err(err) => println!("Not ok: {err}"),
    }
}
