use super::{consts::MAX_BUF_SIZE, error::Result};

use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
};

pub fn diff_pointer(p1: &str, p2: &str) -> Result<()> {
    let p1 = fs::read(p1)?;
    let p2 = fs::read(p2)?;
    let mut out = BufWriter::with_capacity(
        MAX_BUF_SIZE,
        OpenOptions::new()
            .read(true)
            .write(true)
            .append(true)
            .create(true)
            .open("out.bin")?,
    );
    let (magic1, data1) = p1.split_at(20);
    let (mode1, last1) = magic1.split_at(1);
    let (max_depth1, name1) = last1.split_at(1);

    let (magic2, data2) = p2.split_at(20);
    let (mode2, last2) = magic2.split_at(1);
    let (max_depth2, name2) = last2.split_at(1);

    if mode1 == mode2 && max_depth1 == max_depth2 {
        if mode1 == [1] && name1 == name2 {
            let p1 = data1.chunks(32).collect::<HashSet<_>>();
            let p2 = data2.chunks(32).collect::<HashSet<_>>();

            out.write_all(magic1)?;
            for data in p1.intersection(&p2) {
                out.write_all(data)?;
            }
        } else if mode1 == [2] {
            let p1 = data1.chunks(48).collect::<HashSet<_>>();
            let p2 = data2.chunks(48).collect::<HashSet<_>>();

            out.write_all(magic1)?;
            for data in p1.intersection(&p2) {
                out.write_all(data)?;
            }
        } else {
            println!("err")
        }
    } else {
        println!(
            "两次扫描模式不一样 p1:lv{:?}-p2:lv{:?},p1:depth{:?}-p2:depth{:?}",
            mode1, mode2, max_depth1, magic2
        )
    }

    Ok(())
}
