use crate::vmmap::VirtualMemoryWrite;

pub fn write_address<P>(p: P, address: usize, data: &[u8])
where
    P: VirtualMemoryWrite,
{
    p.write_at(address as _, data).unwrap()
}
