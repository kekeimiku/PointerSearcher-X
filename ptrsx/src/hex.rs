use core::{mem::MaybeUninit, slice, str};

const I128_MAX_LEN: usize = 40;

const HEX_DIGITS_LUT: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub struct Buffer([MaybeUninit<u8>; I128_MAX_LEN]);

impl Buffer {
    #[inline]
    pub fn new() -> Buffer {
        let bytes = [MaybeUninit::<u8>::uninit(); I128_MAX_LEN];
        Buffer(bytes)
    }

    #[inline]
    pub fn format<I: Integer>(&mut self, i: I) -> &str {
        i.write(unsafe {
            &mut *(&mut self.0 as *mut [MaybeUninit<u8>; I128_MAX_LEN] as *mut <I as private::Sealed>::Buffer)
        })
    }
}

pub trait Integer: private::Sealed {}

mod private {
    pub trait Sealed: Copy {
        type Buffer: 'static;
        fn write(self, buf: &mut Self::Buffer) -> &str;
    }
}

impl Integer for usize {}
impl Integer for isize {}

impl private::Sealed for usize {
    type Buffer = [MaybeUninit<u8>; I128_MAX_LEN];

    #[inline]
    fn write(self, buf: &mut Self::Buffer) -> &str {
        unsafe {
            let mut n = self;
            if n > 0 {
                let mut curr = (buf.len()) as isize;
                let lut = HEX_DIGITS_LUT.as_ptr();
                let buf_ptr = buf.as_mut_ptr() as *mut u8;
                while n != 0 {
                    curr -= 1;
                    lut.add(n % 16)
                        .copy_to_nonoverlapping(buf_ptr.offset(curr), 1);
                    n /= 16;
                }

                let len = buf.len() - curr as usize;
                let slice = slice::from_raw_parts(buf_ptr.offset(curr), len);
                return str::from_utf8_unchecked(slice);
            }

            "0"
        }
    }
}

impl private::Sealed for isize {
    type Buffer = [MaybeUninit<u8>; I128_MAX_LEN];

    #[inline]
    fn write(self, buf: &mut Self::Buffer) -> &str {
        unsafe {
            let mut curr = (buf.len()) as isize;
            let buf_ptr = buf.as_mut_ptr() as *mut u8;
            let lut = HEX_DIGITS_LUT.as_ptr();
            let is_nonnegative = self >= 0;
            let mut n = if is_nonnegative {
                self
            } else {
                self.checked_abs().unwrap_or_else(|| {
                    curr -= 1;
                    let value = isize::MAX;
                    let rem = value % 16;
                    lut.offset(rem).copy_to_nonoverlapping(buf_ptr.offset(curr), 1);
                    value / 16 + (rem == 15) as isize
                })
            };

            while n != 0 {
                curr -= 1;
                lut.offset(n % 16).copy_to_nonoverlapping(buf_ptr.offset(curr), 1);
                n /= 16;
            }

            if !is_nonnegative {
                curr -= 1;
                buf_ptr.offset(curr).write(b'-');
            }

            let len = (buf.len() as isize - curr) as usize;
            let slice = slice::from_raw_parts(buf_ptr.offset(curr), len);

            str::from_utf8_unchecked(slice)
        }
    }
}
