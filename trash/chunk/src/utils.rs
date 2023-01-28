// convert vec to bytes
#[inline(always)]
pub fn vec_as_bytes<T>(value: &[T]) -> &[u8] {
    let element_size = core::mem::size_of::<T>();
    unsafe { core::slice::from_raw_parts(value.as_ptr() as *const u8, value.len() * element_size) }
}

// convert Vec<u8> read from file to Vec<T>
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
