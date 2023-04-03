pub mod cmd;
pub mod consts;
pub mod create_map;
pub mod pointer_map;
pub mod scanner_map;

use std::path::PathBuf;
use vmmap::VirtualQuery;

#[derive(Default, Clone, Debug, bincode::Encode, bincode::Decode)]
pub struct Map {
    pub start: usize,
    pub end: usize,
    pub size: usize,
    pub is_read: bool,
    pub is_write: bool,
    pub is_exec: bool,
    pub is_stack: bool,
    pub is_heap: bool,
    pub path: Option<PathBuf>,
}

impl core::fmt::Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:#x}-{:#x} {} {}",
            self.start,
            self.end,
            perm(self.is_read, self.is_write, self.is_exec),
            self.path
                .as_ref()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| {
                    if self.is_stack {
                        String::from("[stack]")
                    } else if self.is_heap {
                        String::from("[heap]")
                    } else {
                        String::from("misc")
                    }
                })
        )
    }
}

#[inline]
fn perm(r: bool, w: bool, x: bool) -> String {
    format!("{}{}{}", if r { "r" } else { "-" }, if w { "w" } else { "-" }, if x { "x" } else { "-" })
}

impl<T> From<T> for Map
where
    T: VirtualQuery,
{
    fn from(value: T) -> Self {
        Self {
            start: value.start(),
            end: value.end(),
            size: value.size(),
            is_read: value.is_read(),
            is_write: value.is_write(),
            is_exec: value.is_exec(),
            is_stack: value.is_stack(),
            is_heap: value.is_heap(),
            path: value.path().map(PathBuf::from),
        }
    }
}
