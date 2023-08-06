#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub type Pid = i32;

#[cfg(target_os = "windows")]
pub type Pid = u32;

mod error;
pub use error::Error;

pub mod vmmap64 {
    use std::path::Path;

    #[cfg(target_os = "linux")]
    pub use super::linux::proc64::{Page, Process};
    #[cfg(target_os = "macos")]
    pub use super::macos::proc64::{Page, Process};
    #[cfg(target_os = "windows")]
    pub use super::windows::proc64::{Page, Process};
    use super::{Error, Pid};

    pub trait VirtualMemoryRead {
        fn read_at(&self, buf: &mut [u8], address: usize) -> Result<usize, Error>;
    }

    pub trait VirtualMemoryWrite {
        fn write_at(&self, buf: &[u8], address: usize) -> Result<(), Error>;
    }

    #[cfg(target_os = "macos")]
    pub trait VirtualQueryExt {
        fn tag(&self) -> u32;
        fn is_reserve(&self) -> bool;
        fn path(&self) -> Option<&Path>;
    }

    #[cfg(target_os = "linux")]
    pub trait VirtualQueryExt {
        fn name(&self) -> &str;
    }

    #[cfg(target_os = "windows")]
    pub trait VirtualQueryExt {
        fn path(&self) -> Option<&Path>;
    }

    pub trait VirtualQuery {
        fn start(&self) -> u64;
        fn end(&self) -> u64;
        fn size(&self) -> u64;
        fn is_read(&self) -> bool;
        fn is_write(&self) -> bool;
        fn is_exec(&self) -> bool;
    }

    pub trait ProcessInfo {
        fn pid(&self) -> Pid;
        fn app_path(&self) -> &Path;
        fn get_maps(&self) -> Box<dyn Iterator<Item = Page> + '_>;
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub mod vmmap32 {
    use std::path::Path;

    #[cfg(target_os = "linux")]
    pub use super::linux::proc32::{Page, Process};
    use super::Pid;

    pub trait VirtualMemoryRead {
        type Error: std::error::Error;

        fn read_at(&self, offset: u32, buf: &mut [u8]) -> Result<usize, Self::Error>;
    }

    pub trait VirtualMemoryWrite {
        type Error: std::error::Error;

        fn write_at(&self, offset: u32, buf: &[u8]) -> Result<(), Self::Error>;
    }

    #[cfg(target_os = "linux")]
    pub trait VirtualQueryExt {
        fn name(&self) -> &str;
    }

    #[cfg(target_os = "windows")]
    pub trait VirtualQueryExt {
        fn path(&self) -> Option<&Path>;
    }

    pub trait VirtualQuery: VirtualQueryExt {
        fn start(&self) -> u32;
        fn end(&self) -> u32;
        fn size(&self) -> u32;
        fn is_read(&self) -> bool;
        fn is_write(&self) -> bool;
        fn is_exec(&self) -> bool;
    }

    pub trait ProcessInfo {
        type Error: std::error::Error;

        fn pid(&self) -> Pid;
        fn app_path(&self) -> &Path;
        fn get_maps(&self) -> Box<dyn Iterator<Item = Page> + '_>;
    }
}
