use std::{cell::RefCell, error::Error, ffi, ptr, slice};

thread_local! {
    static LAST_ERROR: RefCell<Option<Box<dyn Error>>> = RefCell::new(None);
}

pub fn set_last_error<E>(err: E)
where
    E: Error + 'static,
{
    LAST_ERROR.with(|prev| {
        *prev.borrow_mut() = Some(Box::new(err));
    });
}

pub fn set_last_boxed_error(err: Box<dyn Error>) {
    LAST_ERROR.with(|prev| {
        *prev.borrow_mut() = Some(err);
    });
}

#[inline]
fn take_last_error() -> Option<Box<dyn Error>> {
    LAST_ERROR.with(|prev| prev.borrow_mut().take())
}

#[no_mangle]
pub extern "C" fn last_error_length() -> ffi::c_int {
    LAST_ERROR.with(|prev| match *prev.borrow() {
        Some(ref err) => err.to_string().len() as ffi::c_int + 1,
        None => 0,
    })
}

#[no_mangle]
pub unsafe extern "C" fn last_error_message(buffer: *mut ffi::c_char, length: ffi::c_int) -> ffi::c_int {
    if buffer.is_null() {
        return -1;
    }

    let last_error = match take_last_error() {
        Some(err) => err,
        None => return 0,
    };

    let error_message = last_error.to_string();

    let buffer = slice::from_raw_parts_mut(buffer as *mut u8, length as usize);

    if error_message.len() >= buffer.len() {
        return -1;
    }

    ptr::copy_nonoverlapping(error_message.as_ptr(), buffer.as_mut_ptr(), error_message.len());
    buffer[error_message.len()] = 0;

    error_message.len() as ffi::c_int
}
