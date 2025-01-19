use std::f64::consts::PI;
use std::path::Path;
use std::time::Duration;

fn main() {
    let filename = Path::new("db.rrd");
    let start = chrono::Utc::now();
    let end = start + chrono::TimeDelta::seconds(300);

    rrd::create(
        filename,
        Duration::from_secs(1),
        start - chrono::TimeDelta::seconds(1),
        false,
        &[],
        None,
        &[
            "DS:sin:GAUGE:10:-1:1",
            "DS:cos:GAUGE:10:-1:1",
            "RRA:AVERAGE:0.5:1:300",
            "RRA:AVERAGE:0.5:5:300",
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
