use std::{cmp::Ordering, io};

use consts::{Address, CHUNK_SIZE, POINTER_SIZE};
use vmmap::{ProcessInfo, VirtualMemoryRead, VirtualQuery};

use super::check::check_region;

pub fn create_pointer_map_helper<W, P>(proc: P, mut out: W) -> io::Result<()>
where
    P: ProcessInfo + VirtualMemoryRead,
    W: io::Write,
{
    let region = proc.get_maps().filter(check_region).collect::<Vec<_>>();

    let scan_region = region.iter().map(|m| (m.start(), m.size())).collect::<Vec<_>>();

    let map = region
        .into_iter()
        .filter_map(|m| Some((m.start(), m.end(), m.path().map(|f| f.to_path_buf())?)))
        .map(|(start, end, path)| format!("{start}-{end}-{}\n", path.to_string_lossy()))
        .collect::<String>();

    let size = map.len();
    out.write_all(&size.to_le_bytes())?;
    out.write_all(map.as_bytes())?;

    create_pointer_map(proc, &scan_region, &mut out)
}

fn create_pointer_map<P, W>(proc: P, region: &[(Address, Address)], mut out: W) -> io::Result<()>
where
    P: VirtualMemoryRead,
    W: io::Write,
{
    let mut buf = [0; CHUNK_SIZE];
    let mut arr = [0; POINTER_SIZE];

    for &(start, size) in region {
        for off in (0..size).step_by(CHUNK_SIZE) {
            let Ok (size) = proc.read_at(start + off, buf.as_mut_slice()) else {
                break;
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
                    out.write_all(&addr.to_le_bytes())?;
                    out.write_all(&out_addr.to_le_bytes())?;
                }
            }
        }
    }

    Ok(())
}
