use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut config = cbindgen::Config::default();    
    config.usize_is_size_t = true;
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(config)
        .with_language(cbindgen::Language::C)
        .with_no_includes()
        .with_sys_include("stddef.h")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("ptrsx_unix.h");
}
