use super::error::Result;

use std::fs;

pub fn show_map_info(path: &str) -> Result<()> {
    let data = fs::read(path)?;
    let (magic, data) = data.split_at(20);
    let (mode, last) = magic.split_at(1);
    let (max_depth, name) = last.split_at(1);
    println!("max_depth: {}", u8::from_le_bytes(max_depth.try_into().unwrap()));
    if mode == [1] {
        let name = std::str::from_utf8(name).unwrap();
        for bin in data.chunks(32) {
            let (name, offset, path) = parse_lv1(bin, name);
            let path = path.map(|s| s.to_string()).collect::<Vec<_>>().join("->");
            println!("{name}+{offset:#x}->{path}");
        }
    } else if mode == [2] {
        for bin in data.chunks(48) {
            let (name, offset, path) = parse_lv2(bin);
            let path = path.map(|s| s.to_string()).collect::<Vec<_>>().join("->");
            println!("{name}+{offset:#x}->{path}");
        }
    }
    Ok(())
}

pub fn parse_lv1<'a>(bin: &'a [u8], name: &'a str) -> (&'a str, u32, impl Iterator<Item = i16> + 'a) {
    let line = bin.rsplitn(2, |&n| n == 101).nth(1).unwrap();
    let (off, path) = line.split_at(4);
    let off = u32::from_le_bytes(off.try_into().unwrap());
    let path = path
        .chunks(2)
        .rev()
        .map(|x| i16::from_le_bytes(x.try_into().unwrap()));
    (name, off, path)
}

pub fn parse_lv2(bin: &[u8]) -> (&str, u32, impl Iterator<Item = i16> + '_) {
    let line = bin.rsplitn(2, |&n| n == 101).nth(1).unwrap();
    let (name, last) = line.split_at(16);
    let name = unsafe { core::str::from_utf8_unchecked(name) };
    let (off, path) = last.split_at(4);
    let off = u32::from_le_bytes(off.try_into().unwrap());
    let path = path
        .chunks(2)
        .rev()
        .map(|x| i16::from_le_bytes(x.try_into().unwrap()));
    (name, off, path)
}
