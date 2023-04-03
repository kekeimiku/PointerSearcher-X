## vmmap

This crate supports reading virtual memory maps from another process as well as reading and writing memory - and supports Linux, macOS and Windows operating systems.

Example:

```rust
use vmmap::{Pid, Process, ProcessInfo, VirtualQuery, VirtualMemoryRead};

let proc = Process::open(pid)?;
let maps = proc.get_maps().collect::<Vec<_>>();
proc.read_at(address, buf.as_mut_slice())?;
```