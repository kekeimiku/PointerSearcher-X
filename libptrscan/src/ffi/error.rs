use core::{
    cell::RefCell,
    ffi::{c_char, c_int},
    ptr,
};
use std::ffi::{CStr, CString};

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) }
}

pub fn set_last_error(err: impl ToString) {
    LAST_ERROR.with(|prev| {
        *prev.borrow_mut() =
            Some(unsafe { CString::from_vec_unchecked(err.to_string().into_bytes()) });
    });
}

/// 如果函数返回值是 int，可以通过 SUCCESS 判断是否调用成功
/// 如果返回的是负数，可以调用 get_last_error 获取具体错误信息
pub const SUCCESS: i32 = 0;
pub(super) const CALL_ERROR: i32 = -1;
pub(super) const API_ERROR: i32 = -2;
pub(super) const NO_NULL: i32 = -3;

// 通常是传入了不允许为空指针的参数
macro_rules! try_null {
    ($m:expr) => {
        match $m {
            Some(p) => p,
            None => return NO_NULL,
        }
    };
}
pub(super) use try_null;

// 通常是触发了内部错误
macro_rules! try_result {
    ($m:expr) => {
        match $m {
            Ok(p) => p,
            Err(err) => {
                set_last_error(err);
                return API_ERROR;
            }
        }
    };
}
pub(super) use try_result;

// 通常是函数调用顺序错了
macro_rules! try_option {
    ($m:expr) => {
        match $m {
            Some(p) => p,
            None => return CALL_ERROR,
        }
    };
}
pub(super) use try_option;

/// 获取具体错误信息
#[no_mangle]
pub unsafe extern "C" fn get_last_error(code: c_int) -> *const c_char {
    if code == API_ERROR {
        LAST_ERROR.with(|prev| match prev.borrow().as_ref() {
            Some(err) => err.as_ptr(),
            None => ptr::null(),
        })
    } else if code == CALL_ERROR {
        // cbindgen 还没支持 c"message" 一类的c字符串...
        const {
            let bytes = concat!("call error", "\0").as_bytes();
            CStr::from_bytes_with_nul_unchecked(bytes).as_ptr()
        }
    } else if code == NO_NULL {
        const {
            let bytes = concat!("param error", "\0").as_bytes();
            CStr::from_bytes_with_nul_unchecked(bytes).as_ptr()
        }
    } else {
        ptr::null()
    }
}
