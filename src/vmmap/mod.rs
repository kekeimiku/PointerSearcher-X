#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use macos::proc::Process;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::proc::Process;

pub mod error;
use error::{Error, Result};

pub trait VirtualMemoryRead {
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize>;
}

pub trait VirtualMemoryWrite {
    fn write_at(&self, offset: usize, buf: &[u8]) -> Result<()>;
}

pub trait VirtualQuery {
    fn start(&self) -> usize;
    fn end(&self) -> usize;
    fn size(&self) -> usize;
    fn is_read(&self) -> bool;
    fn is_write(&self) -> bool;
    fn is_exec(&self) -> bool;
    fn is_stack(&self) -> bool;
    fn is_heap(&self) -> bool;
    fn path(&self) -> &str;
    fn name(&self) -> &str;
}

use core::fmt::Display;

pub trait VirtualMemoryInfo {
    fn get_maps(&self) -> impl Iterator<Item = impl VirtualQuery + Clone + Display + '_>;
}

// pub trait VirtualMemoryInfo {
//     type Item<'a>: VirtualQuery
//     where
//         Self: 'a;
//     type Iter<'a>: Iterator<Item = Self::Item<'a>>
//     where
//         Self: 'a;
//     fn get_maps(&self) -> Self::Iter<'_>;
// }
