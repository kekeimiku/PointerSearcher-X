use std::{
    ffi::OsString,
    fs::File,
    io,
    os::unix::prelude::{FileExt, MetadataExt, OsStringExt},
    path::{Path, PathBuf},
};

use ptrsx::consts::Address;

pub fn convert_bin_to_txt<P: AsRef<Path>, W: io::Write>(path: P, mut out: W) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut seek = 0;

    let mut buf = [0; 24];
    file.read_exact_at(&mut buf, seek)?;
    seek += buf.len() as u64;

    let size = usize::from_le_bytes(buf[8..16].try_into().unwrap());
    let len = usize::from_le_bytes(buf[16..24].try_into().unwrap());

    let mut buf = vec![0; len];
    file.read_exact_at(&mut buf, seek)?;
    seek += buf.len() as u64;

    let pathname = PathBuf::from(OsString::from_vec(buf));

    assert_eq!((file.metadata()?.size() - seek) % size as u64, 0);

    let mut buf = vec![0; size * 1000];

    loop {
        let n = file.read_at(&mut buf, seek)?;
        if n == 0 {
            break;
        }
        seek += n as u64;

        for bin in buf[..n].chunks(size) {
            let (off, path) = wrap_parse_line(bin)?;
            let ptr = path.map(|s| s.to_string()).collect::<Vec<_>>().join("->");
            let name = pathname
                .file_name()
                .and_then(|f| f.to_str())
                .ok_or("get file name error")?;
            writeln!(out, "{name}+{:#x}->{ptr}", off)?;
        }
    }

    Ok(())
}

#[inline(always)]
pub fn wrap_parse_line(bin: &[u8]) -> Result<(Address, impl Iterator<Item = i16> + '_), &'static str> {
    parse_line(bin).ok_or("parse error")
}

#[inline(always)]
fn parse_line(bin: &[u8]) -> Option<(Address, impl Iterator<Item = i16> + '_)> {
    let line = bin.rsplitn(2, |&n| n == 101).nth(1)?;
    let (off, path) = line.split_at(8);
    let off = Address::from_le_bytes(off.try_into().unwrap());
    let path = path.chunks(2).rev().map(|x| i16::from_le_bytes(x.try_into().unwrap()));

    Some((off, path))
}
