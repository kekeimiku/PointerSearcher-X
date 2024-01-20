#[cfg(target_os = "macos")]
#[derive(Debug)]
pub enum Error {
    OpenProcess(machx::kern_return::kern_return_t),
    ReadMemory(machx::kern_return::kern_return_t),
    WriteMemory(machx::kern_return::kern_return_t),
    QueryMapping(machx::kern_return::kern_return_t),
}

#[cfg(any(target_os = "linux", target_os = "android"))]
#[derive(Debug)]
pub enum Error {
    OpenProcess(std::io::Error),
    ReadMemory(std::io::Error),
    WriteMemory(std::io::Error),
    QueryMapping(std::io::Error),
}

#[cfg(target_os = "windows")]
#[derive(Debug)]
pub enum Error {
    OpenProcess(windows_sys::Win32::Foundation::WIN32_ERROR),
    ReadMemory(windows_sys::Win32::Foundation::WIN32_ERROR),
    WriteMemory(windows_sys::Win32::Foundation::WIN32_ERROR),
    QueryMapping(windows_sys::Win32::Foundation::WIN32_ERROR),
}

#[cfg(target_os = "macos")]
#[inline]
pub fn mach_error<'a>(code: i32) -> &'a str {
    unsafe {
        let ptr = machx::error::mach_error_string(code);
        core::str::from_utf8_unchecked(core::ffi::CStr::from_ptr(ptr).to_bytes())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(target_os = "macos")]
        match self {
            Error::OpenProcess(err) => write!(f, "OpenProcess: {}. code: {err}", mach_error(*err)),
            Error::ReadMemory(err) => write!(f, "ReadMemory: {}. code: {err}", mach_error(*err)),
            Error::WriteMemory(err) => write!(f, "WriteMemory: {}. code: {err}", mach_error(*err)),
            Error::QueryMapping(err) => write!(f, "QueryMapping: {}. code: {err}", mach_error(*err)),
        }
        #[cfg(any(target_os = "linux", target_os = "android"))]
        match self {
            Error::OpenProcess(err) => write!(f, "OpenProcess: {err}"),
            Error::ReadMemory(err) => write!(f, "ReadMemory: {err}"),
            Error::WriteMemory(err) => write!(f, "WriteMemory: {err}"),
            Error::QueryMapping(err) => write!(f, "QueryMapping: {err}"),
        }
        #[cfg(target_os = "windows")]
        match self {
            Error::OpenProcess(err) => write!(f, "OpenProcess, code: {err}"),
            Error::ReadMemory(err) => write!(f, "ReadMemory, code: {err}"),
            Error::WriteMemory(err) => write!(f, "WriteMemory, code: {err}"),
            Error::QueryMapping(err) => write!(f, "QueryMapping, code: {err}"),
        }
    }
}

impl std::error::Error for Error {}
