use std::{cmp::Ordering, collections::BTreeMap, io};

use vmmap::vmmap32::VirtualMemoryRead;

use super::DEFAULT_BUF_SIZE;

pub fn create_pointer_map_writer<W, P>(proc: &P, region: &[(u32, u32)], out: &mut W) -> io::Result<()>
where
    W: io::Write,
    P: VirtualMemoryRead,
{
    let mut buf = [0; DEFAULT_BUF_SIZE];

    for &(start, size) in region {
        for off in (0..size).step_by(DEFAULT_BUF_SIZE) {
            let Ok (size) = proc.read_at((start + off) as _, buf.as_mut_slice()) else {
                break;
            };
            for (k, buf) in buf[..size].windows(4).enumerate() {
                let addr = start + off + k as u32;
                let out_addr = u32::from_le_bytes(unsafe { *(buf.as_ptr() as *const _) });
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

pub fn create_pointer_map_btree<W, P>(proc: &P, region: &[(u32, u32)], out: &mut BTreeMap<u32, u32>)
where
    W: io::Write,
    P: VirtualMemoryRead,
{
    let mut buf = [0; DEFAULT_BUF_SIZE];

    for &(start, size) in region {
        for off in (0..size).step_by(DEFAULT_BUF_SIZE) {
            let Ok (size) = proc.read_at((start + off) as _, buf.as_mut_slice()) else {
                break;
            };
            for (k, buf) in buf[..size].windows(4).enumerate() {
                let addr = start + off + k as u32;
                let out_addr = u32::from_le_bytes(unsafe { *(buf.as_ptr() as *const _) });
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
