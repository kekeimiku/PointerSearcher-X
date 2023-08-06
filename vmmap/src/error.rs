#[derive(Debug)]
pub enum Error {
    #[cfg(target_os = "linux")]
    OpenProcess(std::io::Error),
    #[cfg(target_os = "linux")]
    ReadMemory(std::io::Error),
    #[cfg(target_os = "linux")]
    WriteMemory(std::io::Error),
    #[cfg(target_os = "macos")]
    OpenProcess(machx::kern_return::kern_return_t),
    #[cfg(target_os = "macos")]
    ReadMemory(machx::kern_return::kern_return_t),
    #[cfg(target_os = "macos")]
    WriteMemory(machx::kern_return::kern_return_t),
    #[cfg(target_os = "windows")]
    OpenProcess(windows_sys::Win32::Foundation::WIN32_ERROR),
    #[cfg(target_os = "windows")]
    ReadMemory(windows_sys::Win32::Foundation::WIN32_ERROR),
    #[cfg(target_os = "windows")]
    WriteMemory(windows_sys::Win32::Foundation::WIN32_ERROR),
}

#[cfg(target_os = "macos")]
#[inline]
pub unsafe fn mach_error(error_value: machx::error::mach_error_t) -> String {
    let ptr = machx::error::mach_error_string(error_value);
    String::from_utf8_unchecked(std::ffi::CStr::from_ptr(ptr).to_bytes().to_owned())
}

#[cfg(target_os = "macos")]
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            match self {
                Error::OpenProcess(err) => write!(f, "{}. code: {err}", mach_error(*err)),
                Error::ReadMemory(err) => write!(f, "{}. code: {err}", mach_error(*err)),
                Error::WriteMemory(err) => write!(f, "{}. code: {err}", mach_error(*err)),
            }
        }
    }
}

#[cfg(target_os = "linux")]
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::OpenProcess(err) => write!(f, "{err}"),
            Error::ReadMemory(err) => write!(f, "{err}"),
            Error::WriteMemory(err) => write!(f, "{err}"),
        }
    }
}

#[cfg(target_os = "windows")]
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::OpenProcess(err) => write!(f, "OpenProcess, code: {err}"),
            Error::ReadMemory(err) => write!(f, "ReadMemory, code: {err}"),
            Error::WriteMemory(err) => write!(f, "WriteMemory, code: {err}"),
        }
    }
}

impl std::error::Error for Error {}
