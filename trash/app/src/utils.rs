// hex string conversion usize; 0x12345 => 74565;
#[inline(always)]
pub fn hexstr_to_usize<S: AsRef<str>>(value: S) -> Result<usize, ::std::num::ParseIntError> {
    usize::from_str_radix(&value.as_ref().replace("0x", ""), 16)
}

// convert vec to bytes
#[inline(always)]
pub fn vec_as_bytes<T>(value: &[T]) -> &[u8] {
    let element_size = core::mem::size_of::<T>();
    unsafe { core::slice::from_raw_parts(value.as_ptr() as *const u8, value.len() * element_size) }
}

// convert Vec<u8> read from file to Vec<T>, 避免内存重新分配
#[inline(always)]
pub fn vec_from_bytes<T>(value: Vec<u8>) -> Vec<T> {
    let data = value.as_ptr();
    let len = value.len();
    let capacity = value.capacity();
    let element_size = core::mem::size_of::<T>();
    unsafe {
        core::mem::forget(value);
        Vec::from_raw_parts(data as *mut T, len / element_size, capacity / element_size)
    }
}
