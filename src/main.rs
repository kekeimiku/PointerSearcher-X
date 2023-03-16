use ups_cli::{
    cmd::{CommandEnum, Commands},
    dm::diff_pointer,
    pc::write_address,
    ps::init_pointer_scanner,
    sm::show_map_info,
    sp::show_pointer_value,
    vmmap::Process,
};

fn main() -> ups_cli::error::Result<()> {
    let cmds: Commands = argh::from_env();

    match cmds.nested {
        CommandEnum::PointerScanner(args) => {
            let proc = Process::open(args.pid)?;
            init_pointer_scanner(&proc)?;
        }
        CommandEnum::DiffPointerMaps(args) => {
            diff_pointer(&args.f1, &args.f2)?;
        }
        CommandEnum::ShowPointerMaps(args) => show_map_info(&args.f)?,
        CommandEnum::ViewPointerPath(args) => {
            let proc = Process::open(args.pid)?;
            show_pointer_value(&proc, &args.target)?;
        }
        CommandEnum::MemoryScanner(_args) => {
            // let proc = Process::open(args.pid)?;
            // let it = proc.get_maps();
            // println!("todo")
        }
        CommandEnum::PatchMemory(args) => {
            let proc = Process::open(args.pid)?;
            let address = usize::from_str_radix(args.addr.trim_start_matches("0x"), 16)?;
            write_address(proc, address, &args.value.to_le_bytes());
            // println!("todo")
        }
    }
    Ok(())
}
