use std::{env, path::Path};

fn main() {
    println!("cargo:rerun-if-env-changed=LIBRRD");
    if let Ok(ref s) = env::var("LIBRRD") {
        let p = Path::new(s);
        assert!(p.is_file());
        let link_lib = Path::new(p.file_name().expect("no file name in LIBRRD env"))
            .file_stem()
            .unwrap()
            .to_string_lossy();
        #[cfg(any(target_family = "unix", target_os = "macos"))]
        let link_lib = link_lib.strip_prefix("lib").unwrap();
        let link_search = p
            .parent()
            .expect("no library path in LIBRRD env")
            .to_string_lossy();
        println!("cargo:rustc-link-lib={link_lib}");
        println!("cargo:rustc-link-search={link_search}");
    } else {
        #[cfg(any(target_family = "unix", target_os = "macos"))]
        {
            if pkg_config::Config::new()
                .atleast_version("1.5.0")
                .probe("librrd")
                .is_ok()
            {
                return;
            }
        }
        panic!("Could not find librrd");
    }
}
