use std::{env, path::PathBuf};

fn main() {
    if env::var("DOCS_RS").is_ok() {
        // Nothing to do
        return;
    }

    configure_rrd();
    create_bindings();
}

fn configure_rrd() {
    // TODO: LIBRRD based discovery
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

fn create_bindings() {
    let bindings = bindgen::Builder::default()
        .header("src/wrapper.h")
        .allowlist_item("rrd_.*")
        .use_core()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
