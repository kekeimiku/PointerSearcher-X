pub struct Error(
    #[cfg(any(target_os = "macos", target_os = "ios"))] super::apple::Error,
    #[cfg(any(target_os = "linux", target_os = "android"))] super::linux::Error,
);

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}
