use std::{
    fmt::Debug,
    fs, io,
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    path::Path,
    ptr, slice,
};

use memmap2::MmapMut;

pub struct Vec<T> {
    mmap: MmapMut,
    len: usize,
    capacity: usize,
    phantom: PhantomData<T>,
}

impl<T> Vec<T> {
    pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = fs::OpenOptions::new().read(true).write(true).open(path)?;
        #[cfg(target_family = "unix")]
        use std::os::unix::fs::MetadataExt;
        #[cfg(target_family = "unix")]
        let capacity = file.metadata()?.size() as usize / mem::size_of::<T>();
        #[cfg(target_family = "windows")]
        use std::os::windows::fs::MetadataExt;
        #[cfg(target_family = "windows")]
        let capacity = file.metadata()?.file_size() as usize / mem::size_of::<T>();
        let mmap = unsafe { MmapMut::map_mut(&file)? };
        Ok(Self { mmap, len: 0, capacity, phantom: PhantomData })
    }

    #[inline]
    pub fn push(&mut self, value: T) {
        assert!(self.len < self.capacity, "capacity error");
        unsafe { self.as_mut_ptr().add(self.len).write(value) };
        self.len += 1;
    }

    #[inline]
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.retain_mut(|elem| f(elem));
    }

    #[inline]
    pub fn retain_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        let original_len = self.len();
        self.len = 0;
        struct BackshiftOnDrop<'a, T> {
            v: &'a mut Vec<T>,
            processed_len: usize,
            deleted_cnt: usize,
            original_len: usize,
        }

        impl<T> Drop for BackshiftOnDrop<'_, T> {
            fn drop(&mut self) {
                if self.deleted_cnt > 0 {
                    unsafe {
                        ptr::copy(
                            self.v.as_ptr().add(self.processed_len),
                            self.v.as_mut_ptr().add(self.processed_len - self.deleted_cnt),
                            self.original_len - self.processed_len,
                        );
                    }
                }
                self.v.len = self.original_len - self.deleted_cnt
            }
        }

        let mut g = BackshiftOnDrop { v: self, processed_len: 0, deleted_cnt: 0, original_len };

        fn process_loop<F, T, const DELETED: bool>(original_len: usize, f: &mut F, g: &mut BackshiftOnDrop<'_, T>)
        where
            F: FnMut(&mut T) -> bool,
        {
            while g.processed_len != original_len {
                let cur = unsafe { &mut *g.v.as_mut_ptr().add(g.processed_len) };
                if !f(cur) {
                    g.processed_len += 1;
                    g.deleted_cnt += 1;
                    unsafe { ptr::drop_in_place(cur) };
                    if DELETED {
                        continue;
                    } else {
                        break;
                    }
                }
                if DELETED {
                    unsafe {
                        let hole_slot = g.v.as_mut_ptr().add(g.processed_len - g.deleted_cnt);
                        ptr::copy_nonoverlapping(cur, hole_slot, 1);
                    }
                }
                g.processed_len += 1;
            }
        }

        process_loop::<F, T, false>(original_len, &mut f, &mut g);

        process_loop::<F, T, true>(original_len, &mut f, &mut g);

        drop(g);
    }

    #[inline]
    pub fn iter(&self) -> slice::Iter<'_, T> {
        self.deref().iter()
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[allow(clippy::missing_safety_doc)]
    #[inline]
    pub unsafe fn as_ptr(&self) -> *const T {
        self.mmap.as_ptr() as *const T
    }

    #[allow(clippy::missing_safety_doc)]
    #[inline]
    pub unsafe fn as_mut_ptr(&mut self) -> *mut T {
        self.mmap.as_mut_ptr() as *mut T
    }

    #[inline]
    pub fn clear(&mut self) {
        let elems: *mut [T] = self.deref_mut();
        unsafe {
            self.len = 0;
            ptr::drop_in_place(elems);
        }
    }
}

impl<A> Extend<A> for Vec<A> {
    fn extend<T: IntoIterator<Item = A>>(&mut self, iter: T) {
        iter.into_iter().for_each(move |v| self.push(v))
    }
}

impl<T: Debug> Debug for Vec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> Deref for Vec<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
    }
}

impl<T> DerefMut for Vec<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
    }
}

impl<T> Drop for Vec<T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            for i in 0..self.len {
                self.as_mut_ptr().add(i).drop_in_place()
            }
        }
    }
}
