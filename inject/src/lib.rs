#![feature(offset_of)]

mod bindgen;
mod ffi;
mod utils;
mod wrapper;

pub use wrapper::inject;
