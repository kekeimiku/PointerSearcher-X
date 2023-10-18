use super::Error;

#[derive(Debug)]
pub enum VirtProcError {
    OpenProcess(std::io::Error),
    ReadMemory(&'static str),
    WriteMemory(&'static str),
    Snapshot(SnapshotError),
}

impl From<SnapshotError> for VirtProcError {
    fn from(value: SnapshotError) -> Self {
        Self::Snapshot(value)
    }
}

#[derive(Debug)]
pub enum SnapshotError {
    Io(std::io::Error),
    Vm(Error),
}

impl From<std::io::Error> for SnapshotError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<Error> for SnapshotError {
    fn from(value: Error) -> Self {
        Self::Vm(value)
    }
}

impl std::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SnapshotError::Io(err) => write!(f, "{err}"),
            SnapshotError::Vm(err) => write!(f, "{err}"),
        }
    }
}
