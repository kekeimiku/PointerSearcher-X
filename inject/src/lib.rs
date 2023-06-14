#![feature(offset_of)]

pub mod error;

mod bindgen;
mod ffi;
mod utils;
mod wrapper;

pub use wrapper::inject;
