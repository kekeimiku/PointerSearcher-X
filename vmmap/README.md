## vmmap

This crate supports reading virtual memory maps from another process as well as reading and writing memory - and supports Linux, macOS and Windows operating systems.

Example:

- query memory map:

```rust
use vmmap::{Process, ProcessInfo, VirtualQuery};

let proc = Process::open(pid)?;

for m in proc.get_maps().flatten().filter(|x| x.is_read()) {

    #[cfg(target_os = "macos")]
    use vmmap::macos::{ShareMode, UserTag, VirtualQueryExt};
    #[cfg(target_os = "macos")]
    println!(
        "{:x}-{:x} {} {} {:?}",
        m.start(),
        m.end(),
        UserTag::from(m.user_tag()),
        ShareMode::from(m.share_mode()),
        m.name()
    );

    #[cfg(target_os = "linux")]
    use vmmap::linux::VirtualQueryExt;
    #[cfg(target_os = "linux")]
    println!("{:x}-{:x} {} {} {} {:?}", m.start(), m.end(), m.offset(), m.dev(), m.inode(), m.name());

    #[cfg(target_os = "windows")]
    use vmmap::windows::VirtualQueryExt;
    #[cfg(target_os = "windows")]
    println!("{:x}-{:x} {} {} {} {:?}", m.start(), m.end(), m.m_type(), m.m_state(), m.m_protect(), m.name());
}
```

- read/write memory:

```rust
use vmmap::{Process, VirtualMemoryRead, VirtualMemoryWrite};

let proc = Process::open(pid)?;

let mut buf = vec![0_u8; 10];
let addr = 0x1234567;

proc.read_exact_at(&mut buf, addr)?;
proc.write_all_at(&buf, addr)?;

let size = proc.read_at(&mut buf, addr)?;
let size = proc.write_at(&buf, addr)?;
```

- find_process_id_by_name:

```rust
#[cfg(target_os = "macos")]
use vmmap::macos::utils::get_process_list_iter;

#[cfg(target_os = "windows")]
use vmmap::windows::utils::get_process_list_iter;

#[cfg(target_os = "linux")]
use vmmap::linux::utils::get_process_list_iter;

let pids = get_process_list_iter()?
    .filter(|(_, path)| {
        path.file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.contains("Process Name"))
    })
    .map(|(id, _)| id);
```