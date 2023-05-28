use std::{cmp::Ordering, io};

use super::proc64::Process64;

pub fn create_pointer_map<W>(proc: &Process64, region: &[(u64, u64)], out: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    let mut buf = [0; 0x100000];

    for &(start, size) in region {
        for off in (0..size).step_by(0x100000) {
            let Ok (size) = proc.read_at((start + off) as _, buf.as_mut_slice()) else {
                break;
            };
            for (k, buf) in buf[..size].windows(8).enumerate() {
                let addr = start + off + k as u64;
                let out_addr = u64::from_le_bytes(unsafe { *(buf.as_ptr() as *const _) });
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
