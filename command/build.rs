use std::{fs, path::Path};

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let path = Path::new(&out_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("deps");

    let target = std::env::var("TARGET").unwrap();

    // fuck https://github.com/rust-lang/rust/issues/44322
    const LINUX: [&str; 2] = ["x86_64-unknown-linux-gnu", "aarch64-linux-android"];
    if LINUX.contains(&target.as_str()) {
        println!("cargo:rustc-link-arg=-Wl,--allow-multiple-definition");
        println!("cargo:rustc-link-search={}", path.display());
    }

    const APPLE: [&str; 2] = ["aarch64-apple-darwin", "aarch64-apple-ios"];
    if APPLE.contains(&target.as_str()) {
        // fuck https://github.com/rust-lang/rust/issues/124462
        fs::rename(
            path.join("libptrscan.dylib"),
            path.join("this.not.static.libptrscan.dylib"),
        )
        .unwrap();

        println!("cargo:rustc-link-search={}", path.display());
    }
}
