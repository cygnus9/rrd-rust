use std::env;

fn main() {
    if env::var("CARGO_FEATURE_LOCKING_MODE").is_ok() {
        let Ok(version) = env::var("DEP_RRD_VERSION") else {
            panic!("locking_mode feature requires librrd >= 1.9.0, but no version information is available");
        };

        let parts: Vec<u32> = version.split('.').filter_map(|s| s.parse().ok()).collect();
        let locking_available = parts.len() >= 2 && {
            let major = parts[0];
            let minor = parts[1];
            major > 1 || (major == 1 && minor >= 9)
        };

        if !locking_available {
            panic!("locking_mode feature requires librrd >= 1.9.0, but found version {version}");
        }
    }
}
