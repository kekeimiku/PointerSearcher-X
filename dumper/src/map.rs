use std::{ffi::OsStr, io, os::unix::prelude::OsStrExt, path::PathBuf};

#[derive(Default, Clone)]
pub struct Map {
    pub start: usize,
    pub end: usize,
    pub path: PathBuf,
}

#[inline]
pub fn encode_map_to_writer<W: io::Write>(map: Vec<Map>, out: &mut W) -> io::Result<()> {
    let mut tmp = vec![];
    let len_b = map.len().to_le_bytes();
    tmp.extend_from_slice(&len_b);
    for Map { start, end, path } in map.into_iter() {
        let path = path.as_os_str().as_bytes();
        let len_path_b = path.len().to_le_bytes();
        tmp.extend_from_slice(&start.to_le_bytes());
        tmp.extend_from_slice(&end.to_le_bytes());
        tmp.extend_from_slice(&len_path_b);
        tmp.extend_from_slice(path);
    }
    out.write_all(&tmp.len().to_le_bytes())?;
    out.write_all(&tmp)?;
    Ok(())
}

#[inline]
pub fn decode_bytes_to_maps(bytes: &[u8]) -> Vec<Map> {
    let mut i = 0;

    let len = usize::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
    let mut maps = Vec::with_capacity(len);
    i += 8;

    for _ in 0..len {
        let start = usize::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
        i += 8;
        let end = usize::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
        i += 8;
        let len = usize::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
        i += 8;
        let path = PathBuf::from(OsStr::from_bytes(&bytes[i..i + len]));
        i += len;
        maps.push(Map { start, end, path });
    }
    maps
}
