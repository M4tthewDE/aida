use std::{env, path::PathBuf};

fn main() {
    let java_home = env::var("JAVA_HOME").unwrap();

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .derive_default(true)
        .clang_arg(format!("-I{java_home}/include"))
        .clang_arg(format!("-I{java_home}/include/linux"))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .unwrap();

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .unwrap();
}
