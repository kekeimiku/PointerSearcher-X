#![feature(offset_of)]

mod bindgen;
mod error;
mod ffi;
mod utils;
mod wrapper;

pub use wrapper::inject;
