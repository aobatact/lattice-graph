use core::slice;
use std::{
    fmt::Debug,
    mem::{self, ManuallyDrop},
    ops::{Index, IndexMut},
    ptr::NonNull,
};

pub struct Array2D<T> {
    heads: NonNull<*mut [T]>,
    hsize: usize,
}

impl<T> Array2D<T> {
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
            heads: NonNull::new_unchecked(hvec.as_mut_ptr()).cast(),
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
        let s2d = ar.mut_2d();
        for i in 0..h {
            for j in 0..v {
                s2d[i][j] = (&mut f)(i, j);
            }
        }
        ar
    }

    #[inline]
    pub fn h_size(&self) -> usize {
        self.hsize
    }

    #[inline]
    pub fn v_size(&self) -> usize {
        unsafe { self.head_mut() }.len()
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.h_size() * self.v_size()
    }

    #[inline]
    pub fn ref_1d(&self) -> &[T] {
        unsafe {
            let x = self.head_mut();
            let vlen = x.len();
            slice::from_raw_parts(x.get_unchecked(0), self.hsize * vlen)
        }
    }

    #[inline]
    pub fn ref_2d<'a>(&self) -> &[&'a [T]] {
        unsafe { slice::from_raw_parts(self.heads.cast().as_ptr(), self.hsize) }
    }

    #[inline]
    pub fn mut_1d(&mut self) -> &mut [T] {
        unsafe {
            let x = self.head_mut();
            let vlen = x.len();
            slice::from_raw_parts_mut(x.get_unchecked_mut(0), self.hsize * vlen)
        }
    }

    #[inline]
    pub fn mut_2d<'a>(&mut self) -> &mut [&'a mut [T]] {
        unsafe { slice::from_raw_parts_mut(self.heads.cast().as_mut(), self.hsize) }
    }

    pub fn into_raw(self) -> Vec<T> {
        unsafe { self.into_raw_inner(true) }
    }

    ///Dropping the returned vec will drop the values in [`Array2D`].
    ///Be careful not to accidentaly drops the inner values.
    unsafe fn into_raw_inner(&self, drop_heads: bool) -> Vec<T> {
        let hlen = self.hsize;
        let x = self.head_mut();
        let len = hlen * x.len();
        let v_val = Vec::from_raw_parts(x.get_unchecked_mut(0), len, len);
        if drop_heads {
            let v_head = Vec::from_raw_parts(self.heads.as_ptr(), 0, hlen);
            drop(v_head);
        }
        v_val
    }

    #[inline]
    unsafe fn head_mut(&self) -> &mut [T] {
        self.heads
            .as_ref()
            .as_mut()
            .unwrap_or_else(|| std::hint::unreachable_unchecked())
    }
}

impl<T: Clone> Clone for Array2D<T> {
    fn clone(&self) -> Self {
        unsafe {
            let vec = self.into_raw_inner(false);
            let vec = ManuallyDrop::new(vec);
            let vec_c = ManuallyDrop::into_inner(vec.clone());
            Self::from_raw_unchecked(self.h_size(), self.v_size(), vec_c)
        }
    }
}

impl<T: Debug> Debug for Array2D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.ref_2d().fmt(f)
    }
}

impl<T: PartialEq> PartialEq for Array2D<T> {
    fn eq(&self, other: &Self) -> bool {
        self.h_size() == other.h_size() && self.ref_1d() == other.ref_1d()
    }
}

impl<T: PartialEq> Eq for Array2D<T> {}

impl<'a, T> AsRef<[&'a [T]]> for Array2D<T> {
    fn as_ref(&self) -> &[&'a [T]] {
        self.ref_2d()
    }
}

impl<'a, T> AsMut<[&'a mut [T]]> for Array2D<T> {
    fn as_mut(&mut self) -> &mut [&'a mut [T]] {
        self.mut_2d()
    }
}

impl<T> AsRef<[T]> for Array2D<T> {
    fn as_ref(&self) -> &[T] {
        self.ref_1d()
    }
}

impl<T> AsMut<[T]> for Array2D<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.mut_1d()
    }
}

impl<T> Drop for Array2D<T> {
    fn drop(&mut self) {
        drop(unsafe { self.into_raw_inner(true) })
    }
}

impl<T> Index<(usize, usize)> for Array2D<T> {
    type Output = T;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.ref_2d()[index.0][index.1]
    }
}

impl<T> IndexMut<(usize, usize)> for Array2D<T> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.mut_2d()[index.0][index.1]
    }
}

#[cfg(test)]
mod tests {
    use super::Array2D;

    #[test]
    fn gen() {
        let x = Array2D::new(5, 2, |h, v| (h, v));
        for i in 0..5 {
            for j in 0..2 {
                assert_eq!(x.ref_2d()[i][j], (i, j));
            }
        }
    }

    #[test]
    fn ref_1d() {
        let x = Array2D::new(5, 2, |h, v| (h, v));
        for i in 0..5 {
            for j in 0..2 {
                assert_eq!(x.ref_1d()[j + i * 2], (i, j));
            }
        }
    }

    #[test]
    fn gen_zst() {
        let x = Array2D::new(5, 2, |_h, _v| ());
        for i in 0..5 {
            for j in 0..2 {
                assert_eq!(x.ref_2d()[i][j], ());
            }
        }
    }

    #[test]
    fn clone() {
        let x = Array2D::new(5, 2, |h, v| (h, v));
        let mut y = x.clone();
        for i in 0..5 {
            for j in 0..2 {
                let yv = &mut y.mut_2d()[i][j];
                assert_eq!(x.ref_2d()[i][j], *yv);
                *yv = (yv.0 + 1, yv.1);
                assert_ne!(x.ref_2d()[i][j], y.mut_2d()[i][j]);
            }
        }
        drop(x);
        for i in 0..5 {
            for j in 0..2 {
                assert_eq!(y.ref_2d()[i][j], (i + 1, j));
            }
        }
    }
}
