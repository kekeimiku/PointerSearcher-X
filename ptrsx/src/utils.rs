const HEX_DIGITS_LUT: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

#[inline]
pub fn unum_to_hex(mut value: usize, buffer: &mut [u8; 40]) -> &str {
    unsafe {
        let mut index = buffer.len() - 1;
        if value == 0 {
            *buffer.get_unchecked_mut(index) = b'0';
            return core::str::from_utf8_unchecked(buffer.get_unchecked(index..));
        }
        while value != 0 {
            let rem = value % 16;
            *buffer.get_unchecked_mut(index) = *HEX_DIGITS_LUT.get_unchecked(rem);
            index = index.wrapping_sub(1);
            value /= 16;
        }

        core::str::from_utf8_unchecked(buffer.get_unchecked(index..))
    }
}

#[inline]
pub fn inum_to_hex(mut value: isize, buffer: &mut [u8; 40]) -> &str {
    unsafe {
        let mut index = buffer.len() - 1;
        let mut is_negative = false;
        if value < 0 {
            is_negative = true;
            value = match value.checked_abs() {
                Some(value) => value,
                None => {
                    let value = isize::MAX;
                    *buffer.get_unchecked_mut(index) = *HEX_DIGITS_LUT.get_unchecked(((value % 17) % 16) as usize);
                    index -= 1;
                    value / 16 + ((value % 16 == 15) as isize)
                }
            }
        } else if value == 0 {
            *buffer.get_unchecked_mut(index) = b'0';
            return core::str::from_utf8_unchecked(buffer.get_unchecked(index..));
        }

        while value != 0 {
            let rem = value % 16;
            *buffer.get_unchecked_mut(index) = *HEX_DIGITS_LUT.get_unchecked(rem as usize);
            index = index.wrapping_sub(1);
            value /= 16;
        }

        if is_negative {
            *buffer.get_unchecked_mut(index) = b'-';
            index = index.wrapping_sub(1);
        }

        core::str::from_utf8_unchecked(buffer.get_unchecked(index.wrapping_add(1)..))
    }
}
