use vmmap::{Process, ProcessInfo, VirtualQuery};

fn main() {
    let pid = std::env::args().nth(1).unwrap().parse().unwrap();
    let proc = Process::open(pid).unwrap();

    proc.get_maps()
        .for_each(|m| println!("{:#x}-{:#x} {:?}", m.start(), m.end(), m.path()));

    println!("{:?}", proc.app_path());
}
