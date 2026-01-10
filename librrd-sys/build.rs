use std::{env, path::{Path, PathBuf}, process::Command, io::Write};
use tempfile;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(rrdsys_use_pregen)");

    if env::var("DOCS_RS").is_ok() {
        println!("cargo::rustc-cfg=rrdsys_use_pregen");
        return;
    }

    if let Some((location, version)) = configure_rrd() {
        println!("cargo::metadata=version={}", version);
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

fn configure_rrd() -> Option<(HeaderLocation, String)> {
    if let Ok(s) = env::var("LIBRRD") {
        configure_rrd_nonstandard(s)
    } else {
        #[cfg(any(target_family = "unix", target_os = "macos"))]
        {
            if let Ok(lib) = pkg_config::Config::new().probe("librrd") {
                if lib.version.as_str() < "1.5.0" {
                    panic!("librrd version {} is too old, need >= 1.5.0", lib.version);
                }
                return Some((HeaderLocation::StandardLocation, lib.version));
            }
        }
        panic!("Could not find librrd");
    }
}

fn configure_rrd_nonstandard<T: AsRef<Path>>(p: T) -> Option<(HeaderLocation, String)> {
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

    let version = get_rrd_version(&include_path, &link_lib);

    Some((HeaderLocation::NonStandardLocation(include_path.to_owned()), version))
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

fn get_rrd_version(include_path: &Path, link_lib: &str) -> String {
    let link_search = include_path.parent().unwrap().to_string_lossy();
    let include_path = include_path.to_string_lossy();

    let c_code = r#"
#include <stdio.h>
#include <rrd.h>

int main() {
    printf("%s\n", rrd_strversion());
    return 0;
}
"#;

    let mut temp_c = tempfile::Builder::new().suffix(".c").tempfile().unwrap();
    temp_c.write_all(c_code.as_bytes()).unwrap();
    let temp_c_path = temp_c.path();

    let output_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("version_check");

    let mut cmd = Command::new("cc");
    cmd.arg(temp_c_path)
       .arg("-o")
       .arg(&output_path)
       .arg(format!("-l{}", link_lib))
       .arg(format!("-I{}", include_path))
       .arg(format!("-L{}", link_search));

    if !cmd.status().unwrap().success() {
        panic!("Failed to compile version check program");
    }

    let output = Command::new(&output_path).output().unwrap();
    if !output.status.success() {
        panic!("Failed to run version check program");
    }

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}
