// TODO enum all tag
// https://github.com/apple-oss-distributions/xnu/blob/5c2921b07a2480ab43ec66f5b9e41cb872bc554f/osfmk/mach/vm_statistics.h#L489
#[derive(Debug)]
pub enum VmTag {
    Malloc,
    MallocSmall,
    MallocLarge,
    MallocHuge,
    Sbrk,
    Realloc,
    MallocTiny,
    MallocLargeReusable,
    MallocLargeReused,
    Stack,
    MallocNano,
    Dylib,
    Dyld,
    DyldMalloc,
    Other(u32),
}

impl From<u32> for VmTag {
    fn from(user_tag: u32) -> Self {
        match user_tag {
            1 => VmTag::Malloc,
            2 => VmTag::MallocSmall,
            3 => VmTag::MallocLarge,
            4 => VmTag::MallocHuge,
            5 => VmTag::Sbrk,
            6 => VmTag::Realloc,
            7 => VmTag::MallocTiny,
            8 => VmTag::MallocLargeReusable,
            9 => VmTag::MallocLargeReused,
            11 => VmTag::MallocNano,
            30 => VmTag::Stack,
            33 => VmTag::Dylib,
            60 => VmTag::Dyld,
            61 => VmTag::DyldMalloc,
            tag => VmTag::Other(tag),
        }
    }
}
