use std::{
    cell::RefCell,
    error::Error,
    ffi::{c_char, CString},
    fmt::Display,
    ptr,
};

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = RefCell::new(None);
}

pub fn set_last_error<E>(err: E)
where
    E: Error + 'static,
{
    unsafe {
        LAST_ERROR.with(|prev| {
            *prev.borrow_mut() = Some(CString::from_vec_unchecked(err.to_string().into()));
        });
    }
}

#[no_mangle]
pub unsafe extern "C" fn last_error_message() -> *const c_char {
    LAST_ERROR.with(|prev| match prev.borrow().as_ref() {
        Some(err) => err.as_ptr(),
        None => ptr::null_mut(),
    })
}

#[derive(Debug)]
pub struct StrErrorWrap(pub &'static str);
impl Display for StrErrorWrap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Error for StrErrorWrap {}

#[macro_export]
macro_rules! ffi_try_result {
    ($expr:expr, $ret_value:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => {
                super::error::set_last_error(err);
                return $ret_value;
            }
        }
    };
}
