fn main() -> Result<(), Box<dyn std::error::Error>> {
    use ptrsx::sc64::PtrsxScanner;
    let mut p = PtrsxScanner::default();
    p.load_pointer_map("hello.ptrs")?;

    println!("{:?}", p.pages());
    println!("{:?}", p.map.len());
    println!("{:?}", p.into_rev_pointer_map().len());
    Ok(())
}

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     use std::{fs::OpenOptions, io::BufWriter};

//     use ptrsx::c64::default_dump_ptr;
//     use vmmap::vmmap64::Process;
//     let proc = Process::open(14505)?;
//     let file = OpenOptions::new()
//         .append(true)
//         .write(true)
//         .create_new(true)
//         .open("hello.ptrs")?;
//     let mut writer = BufWriter::new(file);
//     default_dump_ptr(&proc, &mut writer)?;
//     Ok(())
// }
