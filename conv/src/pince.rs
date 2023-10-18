#[cfg(target_family = "unix")]
use std::os::unix::fs::MetadataExt;
#[cfg(target_family = "windows")]
use std::os::windows::fs::MetadataExt;
use std::{
    fs,
    io::{BufRead, BufReader, BufWriter},
    path::{Path, PathBuf},
};

use super::{Result, SubCommandPince};

impl SubCommandPince {
    pub fn init(self) -> Result<()> {
        let SubCommandPince { scandata, pct } = self;
        let pct = pct.unwrap_or_else(|| {
            let name = scandata.file_name().and_then(|s| s.to_str()).unwrap_or("unknown");
            PathBuf::from(name).with_extension("pct")
        });
        conv_pince(scandata, pct, "todo")
    }
}

pub fn conv_pince<P: AsRef<Path>>(sd_path: P, pct_path: P, remark: &str) -> Result<()> {
    use std::io::Write;
    let pct_file = fs::OpenOptions::new().append(true).create_new(true).open(pct_path)?;
    let sd_file = fs::File::open(sd_path)?;
    const BUF_SIZE: usize = 0x80000;
    let mut reader = BufReader::with_capacity(BUF_SIZE, sd_file);
    let mut writer = BufWriter::with_capacity(BUF_SIZE, &pct_file);
    write!(writer, "[")?;
    let line_buf = &mut String::with_capacity(0x2000);
    let conv_buf = &mut String::with_capacity(0x2000);
    loop {
        let size = reader.read_line(line_buf)?;
        if size == 0 {
            break;
        }
        let line = conv_line(line_buf.trim_end(), conv_buf, remark).ok_or("parse error")?;
        writeln!(writer, "{line}")?;
        line_buf.clear();
        conv_buf.clear();
    }
    writer.flush()?;
    #[cfg(target_family = "unix")]
    let size = pct_file.metadata()?.size();
    #[cfg(target_family = "windows")]
    let size = pct_file.metadata()?.file_size();
    pct_file.set_len(size - 2)?;
    write!(writer, "]")?;
    writer.flush()?;

    Ok(())
}

#[inline]
pub fn conv_line<'a>(line: &str, conv_buf: &'a mut String, remark: &str) -> Option<&'a str> {
    use std::fmt::Write;
    let mut parts = line.split(['[', ']', '+', '@']).filter(|s| !s.is_empty());
    let name = parts.next()?;
    let index = parts.next()?.parse::<usize>().ok()?;
    let base = parts.next()?.parse::<usize>().ok()?;
    let _ = write!(conv_buf, "[\"{remark}\", [\"{name}[{index}]+0x{base:x}\", [");
    for o in parts.map(|s| s.parse::<isize>()) {
        let o = o.ok()?;
        let _ = write!(conv_buf, "{o},");
    }
    conv_buf.pop();
    let _ = write!(conv_buf, "]], [2, 10, true, 0, 0], []],");
    Some(conv_buf)
}
