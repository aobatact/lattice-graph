use core::slice;
use std::{
    fmt::Debug,
    mem::{self, ManuallyDrop},
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

pub struct Array2<'a, T> {
    heads: NonNull<&'a mut [T]>,
    hsize: usize,
}

impl<'a, T> Array2<'a, T> {
    pub fn from_raw(h: usize, v: usize, vec: Vec<T>) -> Self {
        assert_eq!(h * v, vec.len());
        unsafe { Self::from_raw_unchecked(h, v, vec) }
    }

    unsafe fn from_raw_unchecked(h: usize, v: usize, vec: Vec<T>) -> Self {
        let mut vec = ManuallyDrop::new(vec);
        if mem::size_of::<T>() != 0 {
            vec.shrink_to_fit();
            debug_assert_eq!(vec.len(), vec.capacity());
        }
        let ptr = vec.as_mut_ptr();
        let mut hvec = ManuallyDrop::new(Vec::with_capacity(h));
        let mut pos = 0;
        for _ in 0..h {
            hvec.push(slice::from_raw_parts_mut(ptr.add(pos), v));
            pos += v;
        }
        hvec.shrink_to_fit();
        debug_assert!(mem::size_of::<T>() == 0 || h == hvec.capacity());
        Self {
            heads: NonNull::new_unchecked(hvec.as_mut_ptr()),
            hsize: h,
        }
    }

    pub unsafe fn new_uninit(h: usize, v: usize) -> Self {
        let len = h * v;
        let mut vec = Vec::<T>::with_capacity(len);
        vec.set_len(len);
        Self::from_raw_unchecked(h, v, vec)
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
        unsafe {
            let x = self.heads.as_ref();
            let vlen = x.len();
            slice::from_raw_parts(x.get_unchecked(0), self.hsize * vlen)
        }
    }

    pub fn ref_2d(&self) -> &[&'a [T]] {
        unsafe { slice::from_raw_parts(self.heads.cast().as_ptr(), self.hsize) }
    }

    pub fn mut_1d(&mut self) -> &mut [T] {
        unsafe {
            let x = self.heads.as_mut();
            let vlen = x.len();
            slice::from_raw_parts_mut(x.get_unchecked_mut(0), self.hsize * vlen)
        }
    }

    pub fn mut_2d(&mut self) -> &mut [&'a mut [T]] {
        unsafe { slice::from_raw_parts_mut(self.heads.as_ptr(), self.hsize) }
    }

    pub fn into_raw(self) -> Vec<T> {
        unsafe { self.into_raw_inner(true) }
    }

    unsafe fn into_raw_inner(&self, drop_heads: bool) -> Vec<T> {
        let mut head = self.heads;
        let hlen = self.hsize;
        let x = head.as_mut();
        let vlen = x.len();
        let ptr = x.get_unchecked_mut(0);
        let len = hlen * vlen;
        let v_val = Vec::from_raw_parts(ptr, len, len);
        if drop_heads {
            let v_head = Vec::from_raw_parts(head.as_ptr(), 0, hlen);
            drop(v_head);
        }
        v_val
    }
}

impl<'a, T: Clone> Clone for Array2<'a, T> {
    fn clone(&self) -> Self {
        unsafe {
            let vec = self.into_raw_inner(false);
            let vec = ManuallyDrop::new(vec);
            let vec_c = ManuallyDrop::into_inner(vec.clone());
            Self::from_raw_unchecked(self.h_size(), self.v_size(), vec_c)
        }
    }
}

impl<'a, T: Debug> Debug for Array2<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.ref_2d().fmt(f)
    }
}

impl<'a, T: PartialEq> PartialEq for Array2<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.h_size() == other.h_size() && self.ref_1d() == other.ref_1d()
    }
}

impl<'a, T: PartialEq> Eq for Array2<'a, T> {}

impl<'a, T> AsRef<[&'a [T]]> for Array2<'a, T> {
    fn as_ref(&self) -> &[&'a [T]] {
        self.ref_2d()
    }
}

impl<'a, T> AsMut<[&'a mut [T]]> for Array2<'a, T> {
    fn as_mut(&mut self) -> &mut [&'a mut [T]] {
        self.mut_2d()
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
        self.ref_1d()
    }
}

impl<'a, T> AsMut<[T]> for Array2<'a, T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.mut_1d()
    }
}

impl<'a, T> Drop for Array2<'a, T> {
    fn drop(&mut self) {
        drop(unsafe { self.into_raw_inner(true) })
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
