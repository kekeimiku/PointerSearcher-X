fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    println!("{}", out_dir);
    println!("cargo:rustc-link-search={out_dir}");
}
