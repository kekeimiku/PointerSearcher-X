use std::cmp::Ordering;

use vmmap::VirtualMemoryRead;

use crate::consts::{Address, PointerMap, CHUNK_SIZE, POINTER_SIZE};

pub fn create_pointer_map<P>(proc: &P, region: &[(Address, Address)], out: &mut PointerMap)
where
    P: VirtualMemoryRead,
{
    let mut buf = vec![0; CHUNK_SIZE];
    let mut arr = [0; POINTER_SIZE];

    for &(start, size) in region {
        for off in (0..size).step_by(CHUNK_SIZE) {
            let Ok (size) = proc.read_at(start + off, buf.as_mut_slice()) else {
                // println!("skip {start:#x}-{:#x} read_err",start+size);
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
                    out.insert(addr, out_addr);
                }
            }
        }
    }
}
