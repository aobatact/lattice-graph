use core::slice;
use std::{
    mem,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

pub struct Array2<'a, T> {
    heads: NonNull<&'a mut [T]>,
    hsize: usize,
}

impl<'a, T> Array2<'a, T> {
    pub unsafe fn new_uninit(h: usize, v: usize) -> Self {
        let len = h * v;
        let mut vec = mem::ManuallyDrop::new(Vec::<T>::with_capacity(len));
        if mem::size_of::<T>() != 0 {
            vec.set_len(len);
            vec.shrink_to_fit();
            debug_assert_eq!(vec.len(), vec.capacity());
        }
        let ptr = vec.as_mut_ptr();
        let mut hvec = mem::ManuallyDrop::new(Vec::with_capacity(h));
        let mut pos = 0;
        for _ in 0..h {
            hvec.push(slice::from_raw_parts_mut(ptr.add(pos), v));
            pos += v;
        }
        hvec.shrink_to_fit();
        Self {
            heads: NonNull::new_unchecked(hvec.as_mut_ptr()),
            hsize: h,
        }
    }

    pub fn new<F: FnMut(usize, usize) -> T>(h: usize, v: usize, mut f: F) -> Self {
        let mut ar = unsafe { Self::new_uninit(h, v) };
        for i in 0..h {
            for j in 0..v {
                ar[i][j] = (&mut f)(i, j);
            }
        }
        ar
    }

    pub fn h_size(&self) -> usize {
        self.hsize
    }

    pub fn v_size(&self) -> usize {
        unsafe { self.heads.as_ref() }.len()
    }

    pub fn size(&self) -> usize {
        self.h_size() * self.v_size()
    }

    pub fn ref_1d(&self) -> &[T] {
        self.as_ref()
    }

    pub fn ref_2d(&self) -> &[&'a [T]] {
        self.as_ref()
    }

    pub fn mut_1d(&mut self) -> &mut [T] {
        self.as_mut()
    }

    pub fn mut_2d(&mut self) -> &mut [&'a mut [T]] {
        self.as_mut()
    }
}

impl<'a, T> AsRef<[&'a [T]]> for Array2<'a, T> {
    fn as_ref(&self) -> &[&'a [T]] {
        unsafe { slice::from_raw_parts(self.heads.cast().as_ptr(), self.hsize) }
    }
}

impl<'a, T> AsMut<[&'a mut [T]]> for Array2<'a, T> {
    fn as_mut(&mut self) -> &mut [&'a mut [T]] {
        unsafe { slice::from_raw_parts_mut(self.heads.as_ptr(), self.hsize) }
    }
}

impl<'a, T> Deref for Array2<'a, T> {
    type Target = [&'a mut [T]];
    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.heads.as_ptr(), self.hsize) }
    }
}

impl<'a, T> DerefMut for Array2<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<'a, T> AsRef<[T]> for Array2<'a, T> {
    fn as_ref(&self) -> &[T] {
        unsafe {
            let x = self.heads.as_ref();
            let vlen = x.len();
            slice::from_raw_parts(x.get_unchecked(0), self.hsize * vlen)
        }
    }
}

impl<'a, T> AsMut<[T]> for Array2<'a, T> {
    fn as_mut(&mut self) -> &mut [T] {
        unsafe {
            let x = self.heads.as_mut();
            let vlen = x.len();
            slice::from_raw_parts_mut(x.get_unchecked_mut(0), self.hsize * vlen)
        }
    }
}

impl<'a, T> Drop for Array2<'a, T> {
    fn drop(&mut self) {
        unsafe {
            let mut head = self.heads;
            let hlen = self.hsize;
            let x = head.as_mut();
            let vlen = x.len();
            let ptr = x.get_unchecked_mut(0);
            let len = hlen * vlen;
            let v_val = Vec::from_raw_parts(ptr, len, len);
            drop(v_val);
            let v_head = Vec::from_raw_parts(head.as_ptr(), 0, hlen);
            drop(v_head);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Array2;

    #[test]
    fn gen() {
        let x = Array2::new(5, 2, |h, v| (h, v));
        for i in 0..5 {
            for j in 0..2 {
                assert_eq!(x[i][j], (i, j));
            }
        }
    }

    #[test]
    fn ref_1d() {
        let x = Array2::new(5, 2, |h, v| (h, v));
        for i in 0..5 {
            for j in 0..2 {
                assert_eq!(x.ref_1d()[j + i * 2], (i, j));
            }
        }
    }

    #[test]
    fn gen_zst() {
        let x = Array2::new(5, 2, |_h, _v| ());
        for i in 0..5 {
            for j in 0..2 {
                assert_eq!(x[i][j], ());
            }
        }
    }
}
