use std::path::Path;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let path = Path::new(&out_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    println!("cargo:rustc-link-search={}", path.display());
    println!("cargo:rustc-link-lib=static=ptrscan");
}
