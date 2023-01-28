use super::reader::ReaderExt;

#[derive(Debug)]
pub struct Chunks<T> {
    data: T,
    start: usize,
    size: usize,
    num: usize,
    last: usize,
}

impl<T> Chunks<T> {
    // 计算分块，start:光标开始位置，end:光标结束位置，size:每块多大
    pub fn new(data: T, start: usize, end: usize, mut size: usize) -> Self {
        assert!(end > start, "seek error");

        let mut last = 0;
        let mut num = 1;
        if size < end - start {
            num = (end - start) / size;
            last = (end - start) % size;
        } else {
            size = end - start;
        }

        Self { data, start, size, num, last }
    }
}

impl<T: ReaderExt> Iterator for Chunks<T> {
    type Item = std::io::Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.num != 0 {
            match self.data.read_at(self.start, self.size) {
                Ok(chunk) => {
                    self.start += self.size;
                    self.num -= 1;
                    return Some(Ok(chunk));
                }
                Err(err) => return Some(Err(err)),
            };
        }

        if self.last != 0 {
            match self.data.read_at(self.start, self.last) {
                Ok(chunk) => {
                    self.last = 0;
                    return Some(Ok(chunk));
                }
                Err(err) => return Some(Err(err)),
            };
        }

        None
    }
}
