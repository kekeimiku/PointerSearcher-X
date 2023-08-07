use std::{cmp::Ordering, collections::BTreeMap, io, mem};

use vmmap::VirtualMemoryRead;

use super::*;

pub fn create_pointer_map<P>(proc: &P, region: &[(usize, usize)]) -> Result<BTreeMap<usize, usize>, Error>
where
    P: VirtualMemoryRead,
    Error: From<P::Error>,
{
    let mut buf = [0; DEFAULT_BUF_SIZE];
    let mut map = BTreeMap::new();

    for &(start, size) in region {
        for off in (0..size).step_by(DEFAULT_BUF_SIZE) {
            let size = proc.read_at(buf.as_mut_slice(), start + off)?;
            for (k, buf) in buf[..size].windows(mem::size_of::<usize>()).enumerate() {
                let value = usize::from_le_bytes(unsafe { *(buf.as_ptr().cast()) });
                if region
                    .binary_search_by(|&(start, size)| {
                        if (start..start + size).contains(&value) {
                            Ordering::Equal
                        } else {
                            start.cmp(&value)
                        }
                    })
                    .is_ok()
                {
                    let key = start + off + k;
                    map.insert(key, value);
                }
            }
        }
    }

    Ok(map)
}

pub fn create_pointer_map_with_writer<P, W>(proc: &P, region: &[(usize, usize)], writer: &mut W) -> Result<(), Error>
where
    P: VirtualMemoryRead,
    W: io::Write,
    Error: From<P::Error>,
{
    let mut buf = [0; DEFAULT_BUF_SIZE];

    for &(start, size) in region {
        for off in (0..size).step_by(DEFAULT_BUF_SIZE) {
            let Ok(size) = proc.read_at(buf.as_mut_slice(), start + off) else {
                println!("skip! [read_err: {start:x}]");
                break;
            };
            for (k, buf) in buf[..size].windows(mem::size_of::<usize>()).enumerate() {
                let value = usize::from_le_bytes(unsafe { *(buf.as_ptr().cast()) });
                if region
                    .binary_search_by(|&(start, size)| {
                        if (start..start + size).contains(&value) {
                            Ordering::Equal
                        } else {
                            start.cmp(&value)
                        }
                    })
                    .is_ok()
                {
                    let key = start + off + k;
                    writer.write_all(&key.to_le_bytes())?;
                    writer.write_all(&value.to_le_bytes())?;
                }
            }
        }
    }

    Ok(())
}
