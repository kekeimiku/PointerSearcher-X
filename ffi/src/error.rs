use std::{
    cell::RefCell,
    ffi::{c_char, CString},
    ptr,
};

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = RefCell::new(None);
}

pub fn set_last_error(err: impl ToString) {
    unsafe {
        LAST_ERROR.with(|prev| {
            *prev.borrow_mut() = Some(CString::from_vec_unchecked(err.to_string().into()));
        });
    }
}

#[no_mangle]
pub unsafe extern "C" fn get_last_error() -> *const c_char {
    LAST_ERROR.with(|prev| match prev.borrow().as_ref() {
        Some(err) => err.as_ptr(),
        None => ptr::null_mut(),
    })
}

#[no_mangle]
pub extern "C" fn clear_last_error() {
    LAST_ERROR.with(|prev| *prev.borrow_mut() = None)
}

#[macro_export]
macro_rules! ffi_try_result {
    ($expr:expr, $ret:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => {
                super::set_last_error(err);
                return $ret;
            }
        }
    };
}
