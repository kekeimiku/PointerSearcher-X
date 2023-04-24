use std::{ffi::OsStr, io, mem, os::unix::prelude::OsStrExt, path::PathBuf};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Map {
    pub start: usize,
    pub end: usize,
    pub path: PathBuf,
}

#[allow(clippy::transmute_num_to_bytes)]
#[inline]
pub fn encode_map_to_writer<W: io::Write>(map: Vec<Map>, out: &mut W) -> io::Result<()> {
    unsafe {
        let mut tmp = vec![];
        let len_b = mem::transmute::<usize, [u8; 8]>(map.len());
        tmp.extend_from_slice(&len_b);
        for Map { start, end, path } in map.into_iter() {
            let path = path.as_os_str().as_bytes();
            tmp.extend_from_slice(&mem::transmute::<[usize; 3], [u8; 24]>([start, end, path.len()]));
            tmp.extend_from_slice(path);
        }
        out.write_all(&mem::transmute::<usize, [u8; 8]>(tmp.len()))?;
        out.write_all(&tmp)?;
    }
    Ok(())
}

#[inline]
pub fn decode_bytes_to_maps(bytes: &[u8]) -> Vec<Map> {
    unsafe {
        let mut i = 0;
        let len = mem::transmute::<[u8; 8], usize>(*(bytes.as_ptr() as *const _));
        let mut maps = Vec::with_capacity(len);
        i += 8;

        for _ in 0..len {
            let start = mem::transmute::<[u8; 8], usize>(*(bytes.as_ptr().add(i) as *const _));
            i += 8;
            let end = mem::transmute::<[u8; 8], usize>(*(bytes.as_ptr().add(i) as *const _));
            i += 8;
            let len = mem::transmute::<[u8; 8], usize>(*(bytes.as_ptr().add(i) as *const _));
            i += 8;
            let path = PathBuf::from(OsStr::from_bytes(bytes.get_unchecked(i..i + len)));
            i += len;
            maps.push(Map { start, end, path });
        }

        maps
    }
}

#[test]
fn test_decode_and_encode_map() {
    let map = vec![
        Map { start: 1, end: 2, path: PathBuf::from("value") },
        Map { start: 4, end: 7, path: PathBuf::from("va lue") },
    ];
    let m1 = map.clone();
    let mut out = vec![];
    encode_map_to_writer(map, &mut out).unwrap();

    let d = decode_bytes_to_maps(&out[8..]);

    assert_eq!(d, m1)
}
