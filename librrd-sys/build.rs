use std::{
    env,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

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
        match pkg_config::Config::new()
            .atleast_version("1.5.0")
            .probe("librrd")
        {
            Ok(lib) => {
                println!("cargo::metadata=version={}", lib.version);
                return Some(HeaderLocation::StandardLocation);
            }
            Err(err) => {
                panic!(
                    "Could not find librrd with pkg-config. Install the RRDtool development \
                     package, or set LIBRRD to the full path of the librrd library. pkg-config \
                     error: {err}"
                );
            }
        }

        #[cfg(not(any(target_family = "unix", target_os = "macos")))]
        panic!("Could not find librrd. Set LIBRRD to the full path of the librrd library.");
    }
}

fn configure_rrd_nonstandard<T: AsRef<Path>>(p: T) -> Option<HeaderLocation> {
    let p = p.as_ref();

    // First setup the linker configuration
    assert!(
        p.is_file(),
        "LIBRRD must point to a librrd library file, but '{}' is not a file",
        p.display()
    );
    let file_name = p.file_name().unwrap_or_else(|| {
        panic!(
            "LIBRRD must point to a librrd library file, but '{}' has no file name",
            p.display()
        )
    });
    let link_lib = Path::new(file_name)
        .file_stem()
        .unwrap_or_else(|| {
            panic!(
                "LIBRRD must point to a librrd library file, but '{}' has no file stem",
                p.display()
            )
        })
        .to_string_lossy();
    #[cfg(any(target_family = "unix", target_os = "macos"))]
    let link_lib = link_lib.strip_prefix("lib").unwrap_or_else(|| {
        panic!(
            "LIBRRD library file '{}' should be named like librrd.so or librrd.dylib",
            p.display()
        )
    });
    let link_search = p
        .parent()
        .unwrap_or_else(|| {
            panic!(
                "LIBRRD must point to a librrd library file, but '{}' has no parent directory",
                p.display()
            )
        })
        .to_string_lossy();
    println!("cargo:rustc-link-lib={link_lib}");
    println!("cargo:rustc-link-search={link_search}");

    // Then see if we can find a header file for bindgen
    let include_path = p.parent().expect("checked above");
    if !include_path.join("rrd.h").is_file() {
        eprintln!(
            "LIBRRD was found at '{}', but '{}' does not exist; using pregenerated bindings",
            p.display(),
            include_path.join("rrd.h").display()
        );
        return None;
    }

    // Try to get the version to confirm it works
    let version = get_rrd_version(link_lib, include_path);
    println!("cargo::metadata=version={}", version);

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
    } else {
        let library = pkg_config::probe_library("librrd")
            .expect("Could not find librrd with pkg-config while generating bindings");
        builder = builder.clang_args(
            library
                .include_paths
                .iter()
                .map(|path| format!("-I{}", path.to_string_lossy())),
        );
    }
    let bindings = builder.generate().expect(
        "Unable to generate librrd bindings. Make sure rrd.h is available and clang can parse it.",
    );

    let out_path = PathBuf::from(env::var("OUT_DIR").expect("Cargo should set OUT_DIR"));
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write generated librrd bindings to OUT_DIR");
}

fn get_rrd_version(link_lib: &str, link_search: &Path) -> String {
    let c_code = r#"
#include <stdio.h>

extern const char* rrd_strversion();

int main() {
    printf("%s\n", rrd_strversion());
    return 0;
}
"#;

    let mut temp_c = tempfile::Builder::new()
        .suffix(".c")
        .tempfile()
        .expect("Failed to create temporary C file for librrd version check");
    temp_c
        .write_all(c_code.as_bytes())
        .expect("Failed to write temporary C file for librrd version check");
    let temp_c_path = temp_c.path();

    let output_path =
        PathBuf::from(env::var("OUT_DIR").expect("Cargo should set OUT_DIR")).join("version_check");

    let mut cmd = Command::new("cc");
    cmd.arg(temp_c_path)
        .arg("-o")
        .arg(&output_path)
        .arg(format!("-L{}", link_search.to_string_lossy()))
        .arg(format!("-l{link_lib}"));

    let status = cmd
        .status()
        .expect("Failed to run C compiler for librrd version check");
    if !status.success() {
        panic!(
            "Failed to compile librrd version check program with library search path '{}'",
            link_search.display()
        );
    }

    let output = Command::new(&output_path)
        .output()
        .expect("Failed to run librrd version check program");
    if !output.status.success() {
        panic!(
            "Failed to run librrd version check program: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}
