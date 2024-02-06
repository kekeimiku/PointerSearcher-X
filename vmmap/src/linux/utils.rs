use std::{fs, io, path::PathBuf};

pub fn get_process_list_iter() -> Result<impl Iterator<Item = (i32, PathBuf)>, io::Error> {
    let dirs = fs::read_dir("/proc")?;
    let iter = dirs.flatten().flat_map(|f| {
        let id = f.file_name().to_str()?.parse().ok()?;
        let path = fs::read_link(f.path().join("exe")).ok()?;
        Some((id, path))
    });
    Ok(iter)
}
