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

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Error::OpenProcess(e) => format!("OpenProcess {e}"),
            Error::ReadMemory(e) => format!("OpenProcess {e}"),
            Error::WriteMemory(e) => format!("WriteMemory {e}"),
        };

        write!(f, "{}", s)
    }
}

impl std::error::Error for Error {}
