#[derive(Debug)]
pub enum Error {
    #[cfg(target_os = "linux")]
    OpenProcess(std::io::Error),
    #[cfg(target_os = "linux")]
    ReadMemory(std::io::Error),
    #[cfg(target_os = "linux")]
    WriteMemory(std::io::Error),
    #[cfg(target_os = "macos")]
    OpenProcess(mach2::kern_return::kern_return_t),
    #[cfg(target_os = "macos")]
    ReadMemory(mach2::kern_return::kern_return_t),
    #[cfg(target_os = "macos")]
    WriteMemory(mach2::kern_return::kern_return_t),
    #[cfg(target_os = "windows")]
    OpenProcess(windows_sys::Win32::Foundation::WIN32_ERROR),
    #[cfg(target_os = "windows")]
    ReadMemory(windows_sys::Win32::Foundation::WIN32_ERROR),
    #[cfg(target_os = "windows")]
    WriteMemory(windows_sys::Win32::Foundation::WIN32_ERROR),
}
