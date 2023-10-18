#[derive(Debug)]
pub enum Error {
    ParseInt(std::num::ParseIntError),
    Utf8(std::str::Utf8Error),
    InvalidString,
}

impl From<std::num::ParseIntError> for Error {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::ParseInt(value)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::Utf8(value)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ParseInt(e) => write!(f, "{e}"),
            Error::Utf8(e) => write!(f, "{e}"),
            Error::InvalidString => write!(f, "invalid string"),
        }
    }
}

pub struct Aob<'a> {
    value: &'a Value,
    haystack: &'a [u8],
    len: usize,
}

pub struct Value {
    sigs: Vec<u8>,
    masks: Vec<bool>,
    start: usize,
    sig: u8,
    mask: bool,
}

impl<'a> Aob<'a> {
    #[inline]
    pub fn from_hex_string(needle: &str) -> Result<Value, Error> {
        if needle.len() % 2 != 0 || needle.len() < 8 {
            return Err(Error::InvalidString);
        }
        let mut sigs = Vec::with_capacity(needle.len());
        let mut masks = Vec::with_capacity(needle.len());
        for byte in needle.as_bytes().chunks_exact(2) {
            if byte == b"??" {
                masks.push(false);
                sigs.push(0);
            } else {
                let str = std::str::from_utf8(byte)?;
                let byte = u8::from_str_radix(str, 16)?;
                masks.push(true);
                sigs.push(byte);
            }
        }
        let mut start = masks.iter().take_while(|&&x| !x).count();
        let end = masks.iter().rev().take_while(|&&x| !x).count();
        if start != masks.len() {
            sigs.truncate(sigs.len() - end);
            sigs.drain(..start);
            masks.truncate(masks.len() - end);
            masks.drain(..start);
        } else {
            start = 0;
        }
        let sig = sigs[0];
        let mask = masks[0];

        Ok(Value { sigs, masks, start, sig, mask })
    }

    #[inline]
    pub fn new(haystack: &'a [u8], value: &'a Value) -> Self {
        let len = haystack.len() - value.sigs.len();
        Self { value, haystack, len }
    }

    #[inline]
    pub fn find_iter(&'a self) -> impl Iterator<Item = usize> + 'a {
        Iter { aob: self, pos: 0 }
    }

    #[inline]
    fn compare_byte_array(&self, haystack: &[u8]) -> bool {
        self.value
            .sigs
            .iter()
            .zip(&self.value.masks)
            .enumerate()
            .all(|(k, (&sig, mask))| !mask || haystack[k] == sig)
    }
}

struct Iter<'a> {
    aob: &'a Aob<'a>,
    pos: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let aob = self.aob;
        (self.pos..=aob.len).find_map(|i| {
            self.pos = i + 1;
            if (aob.haystack[i] == aob.value.sig || !aob.value.mask) && aob.compare_byte_array(&aob.haystack[i..]) {
                Some(i - aob.value.start)
            } else {
                None
            }
        })
    }
}
