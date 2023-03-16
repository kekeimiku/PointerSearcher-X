#[derive(Debug)]
pub enum Error {
    #[cfg(target_os = "linux")]
    OpenProcess(std::io::Error),
    #[cfg(target_os = "linux")]
    ReadMemory(std::io::Error),
    #[cfg(target_os = "linux")]
    WriteMemory(std::io::Error),
    #[cfg(target_os = "macos")]
    OpenProces(super::macos::ffi::kern_return_t),
    #[cfg(target_os = "macos")]
    ReadMemory(super::macos::ffi::kern_return_t),
    #[cfg(target_os = "macos")]
    WriteMemory(super::macos::ffi::kern_return_t),
}

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[cfg(target_os = "linux")]
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::OpenProcess(value)
    }
}

#[cfg(target_os = "macos")]
impl From<super::macos::ffi::kern_return_t> for Error {
    fn from(value: i32) -> Self {
        Self::OpenProces(value)
    }
}
