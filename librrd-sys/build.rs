use std::{env, path::{Path, PathBuf}};

fn main() {
    println!("cargo::rustc-check-cfg=cfg(rrdsys_use_pregen)");

    if env::var("DOCS_RS").is_ok() {
        println!("cargo::rustc-cfg=rrdsys_use_pregen");
        return;
    }

    if let Some(location) = configure_rrd() {
        create_bindings(location);
    } else {
        println!("cargo::rustc-cfg=rrdsys_use_pregen");
    }
}

#[allow(dead_code)]
enum HeaderLocation {
    NonStandardLocation(PathBuf),
    StandardLocation,
}

fn configure_rrd() -> Option<HeaderLocation> {
    if let Ok(s) = env::var("LIBRRD") {
        configure_rrd_nonstandard(s)
    } else {
        #[cfg(any(target_family = "unix", target_os = "macos"))]
        {
            if pkg_config::Config::new()
                .atleast_version("1.5.0")
                .probe("librrd")
                .is_ok()
            {
                return Some(HeaderLocation::StandardLocation);
            }
        }
        panic!("Could not find librrd");
    }
}

fn configure_rrd_nonstandard<T: AsRef<Path>>(p: T) -> Option<HeaderLocation> {
    let p = p.as_ref();

    // First setup the linker configuration
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

    // Then see if we can find a header file for bindgen
    let include_path = p.parent().unwrap();
    if !include_path.join("rrd.h").is_file() {
        return None;
    }

    Some(HeaderLocation::NonStandardLocation(include_path.to_owned()))
}

fn create_bindings(location: HeaderLocation) {
    let mut builder = bindgen::Builder::default()
        .header("src/gen/wrapper.h")
        .allowlist_item("rrd_.*")
        .use_core()
        .opaque_type("_IO_FILE")     // Treat as opaque - we only use FILE*, never sizeof(FILE)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));
    if let HeaderLocation::NonStandardLocation(location) = location {
        builder = builder.clang_arg(format!("-I{}", location.to_string_lossy()));
    }
    let bindings = builder
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
