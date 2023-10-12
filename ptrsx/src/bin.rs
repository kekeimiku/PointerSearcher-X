use std::{io, io::Write, mem};

use super::*;

pub fn encode_modules<W: io::Write>(pages: &[Module], writer: &mut W) -> io::Result<()> {
    let mut data = Vec::new();
    let len = pages.len().to_le_bytes();
    data.write_all(&len)?;
    for Module { start, end, name: path } in pages {
        data.write_all(&start.to_le_bytes())?;
        data.write_all(&end.to_le_bytes())?;
        data.write_all(&path.len().to_le_bytes())?;
        data.write_all(path.as_bytes())?;
    }
    #[cfg(target_pointer_width = "64")]
    writer.write_all(&PTRHEADER64)?;
    #[cfg(target_pointer_width = "32")]
    writer.write_all(&PTRHEADER32)?;
    writer.write_all(&data.len().to_le_bytes())?;
    writer.write_all(&data)
}

pub fn decode_modules(bytes: &[u8]) -> Vec<Module> {
    const SIZE: usize = mem::size_of::<usize>();
    unsafe {
        let mut seek = 0;
        let len = usize::from_le_bytes(*(bytes.as_ptr().cast()));
        let mut pages = Vec::with_capacity(len);
        seek += SIZE;
        (0..len).for_each(|_| {
            let start = usize::from_le_bytes(*(bytes.as_ptr().add(seek).cast()));
            seek += SIZE;
            let end = usize::from_le_bytes(*(bytes.as_ptr().add(seek).cast()));
            seek += SIZE;
            let len = usize::from_le_bytes(*(bytes.as_ptr().add(seek).cast()));
            seek += SIZE;
            let name = String::from_utf8_unchecked(bytes.get_unchecked(seek..seek + len).to_vec());
            seek += len;
            pages.push(Module { start, end, name })
        });
        pages
    }
}
