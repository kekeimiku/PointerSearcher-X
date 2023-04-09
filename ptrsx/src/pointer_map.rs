use std::{cmp::Ordering, collections::BTreeMap, io, path::PathBuf};

use vmmap::{ProcessInfo, VirtualMemoryRead, VirtualQuery};

use crate::{
    check::check_region,
    consts::{Address, BIN_CONFIG, CHUNK_SIZE, POINTER_SIZE},
    error::Result,
};

pub fn ptrsx_create_pointer_map<W, P>(proc: P, mut p_out: W, mut m_out: W) -> Result<()>
where
    P: ProcessInfo + VirtualMemoryRead,
    W: io::Write,
{
    let region = proc
        .get_maps()
        .filter(|m| m.is_read())
        .filter(check_region)
        .collect::<Vec<_>>();

    let scan_region = region.iter().map(|m| (m.start(), m.size())).collect::<Vec<_>>();
    let base_region = region
        .into_iter()
        .filter_map(|m| Some((m.start(), m.end(), m.path().map(|f| f.to_path_buf())?)))
        .collect::<Vec<_>>();

    bincode::encode_into_std_write(base_region, &mut m_out, BIN_CONFIG)?;

    let mut out = BTreeMap::new();

    create_pointer_map(&proc, &scan_region, &mut out);

    bincode::encode_into_std_write(out, &mut p_out, BIN_CONFIG)?;

    Ok(())
}

pub fn ptrsx_decode_maps<R: io::Read>(mut read: R) -> Result<Vec<(Address, Address, PathBuf)>> {
    Ok(bincode::decode_from_std_read(&mut read, BIN_CONFIG)?)
}

fn create_pointer_map<P>(proc: &P, region: &[(Address, Address)], out: &mut BTreeMap<Address, Address>)
where
    P: VirtualMemoryRead,
{
    let mut buf = vec![0; CHUNK_SIZE];
    let mut arr = [0; POINTER_SIZE];

    'inner: for &(start, size) in region {
        for off in (0..size).step_by(CHUNK_SIZE) {
            let Ok (size) = proc.read_at(start + off, buf.as_mut_slice()) else {
                println!(" skip {start:#x}-{:#x} read_err",start+size);
                break 'inner;
            };
            for (o, buf) in buf[..size].windows(POINTER_SIZE).enumerate() {
                let addr = start + off + o;
                arr[0..POINTER_SIZE].copy_from_slice(buf);
                let out_addr = Address::from_le_bytes(arr);
                if region
                    .binary_search_by(|&(a, s)| {
                        if out_addr >= a && out_addr < a + s {
                            Ordering::Equal
                        } else {
                            a.cmp(&out_addr)
                        }
                    })
                    .is_ok()
                {
                    out.insert(addr, out_addr);
                }
            }
        }
    }
}
