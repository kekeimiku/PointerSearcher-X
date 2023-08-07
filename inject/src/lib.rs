cfg_if::cfg_if! {
    if #[cfg(all(target_os = "macos", target_arch = "aarch64"))] {
        mod error;
        mod ffi;
        mod utils;
        mod wrapper;
        pub use wrapper::inject;
    }else {
        panic!("not support.");
    }
}
