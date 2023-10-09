use std::{
    env,
    fs::OpenOptions,
    io::{BufWriter, Write},
};

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let mut file = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open("ptrsx.h")
            .unwrap(),
    );
    file.write_all(b"#include <stddef.h>\n\n").unwrap();
    // bindgen cannot handle type alias.
    file.write_all(b"typedef int Pid;\n\n").unwrap();

    let mut config = cbindgen::Config::default();
    config.usize_is_size_t = true;
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(config)
        .with_language(cbindgen::Language::C)
        .with_no_includes()
        .generate()
        .expect("Unable to generate bindings")
        .write(file)
}
