use core::slice;
use std::{
    fmt::Debug,
    hash::Hash,
    mem::{self, ManuallyDrop, MaybeUninit},
    num::NonZeroUsize,
    ops::{Index, IndexMut},
    ptr::NonNull,
};

/// Raw fixed 2d vector. It has two axis, horizontal and vertical. Data is stored on one [`Vec`].
/// Horizontal size must not be 0.
pub struct FixedVec2D<T> {
    heads: NonNull<*mut [T]>,
    hsize: NonZeroUsize,
}

impl<T> FixedVec2D<T> {
    /// Creates a array2d with a vec.
    /// Returns [`None`] if `h * v != vec.len()`
    pub fn from_raw(h: NonZeroUsize, v: usize, vec: Vec<T>) -> Option<Self> {
        if h.get() * v != vec.len() {
            None
        } else {
            Some(unsafe { Self::from_raw_unchecked(h, v, vec) })
        }
    }

    unsafe fn from_raw_unchecked(h: NonZeroUsize, v: usize, vec: Vec<T>) -> Self {
        let mut vec = ManuallyDrop::new(vec);
        if mem::size_of::<T>() != 0 {
            vec.shrink_to_fit();
            debug_assert_eq!(vec.len(), vec.capacity());
        }
        let ptr = vec.as_mut_ptr();
        let mut hvec = ManuallyDrop::new(Vec::with_capacity(h.get()));
        let mut pos = 0;
        for _ in 0..h.get() {
            hvec.push(slice::from_raw_parts_mut(ptr.add(pos), v));
            pos += v;
        }
        hvec.shrink_to_fit();
        debug_assert!(mem::size_of::<T>() == 0 || h.get() == hvec.capacity());
        Self {
            heads: NonNull::new_unchecked(hvec.as_mut_ptr()).cast(),
            hsize: h,
        }
    }

    /// Creates a FixedVec2D without initializing.
    /// It is unsafe to use it so I recomend to use with [`MaybeUninit`] or just use [`new`](`Self::new`).
    /// See [`assume_init`](`Self::assume_init`).
    pub unsafe fn new_uninit(h: NonZeroUsize, v: usize) -> Self {
        let len = h.get() * v;
        let mut vec = Vec::<T>::with_capacity(len);
        vec.set_len(len);
        Self::from_raw_unchecked(h, v, vec)
    }

    /**
    Creates a FixedVec2D with initializing from the function.
    ```
    # use std::num::NonZeroUsize;
    # use lattice_graph::fixedvec2d::FixedVec2D;
    let array = FixedVec2D::new(NonZeroUsize::new(5).unwrap(), 2, |h, v| (h, v));
    for i in 0..5 {
        for j in 0..2 {
            assert_eq!(array.ref_2d()[i][j], (i, j));
        }
    }
    ```
    */
    pub fn new<F: FnMut(usize, usize) -> T>(h: NonZeroUsize, v: usize, mut f: F) -> Self {
        let mut ar = unsafe { Self::new_uninit(h, v) };
        let s2d = ar.mut_2d();
        for i in 0..h.get() {
            for j in 0..v {
                s2d[i][j] = (&mut f)(i, j);
            }
        }
        ar
    }

    /// Returns the horizontal size.
    #[inline]
    pub fn h_size(&self) -> usize {
        self.hsize.get()
    }

    /// Returns the vertical size.
    #[inline]
    pub fn v_size(&self) -> usize {
        unsafe { self.head_mut() }.len()
    }

    /// Returns the length of underlying [`Vec`].
    #[inline]
    pub fn size(&self) -> usize {
        self.h_size() * self.v_size()
    }

    /// Returns the slice of all values in the array.
    #[inline]
    pub fn ref_1d(&self) -> &[T] {
        unsafe {
            let x = self.head_mut();
            let vlen = x.len();
            slice::from_raw_parts(x.as_ptr(), self.h_size() * vlen)
        }
    }

    /// Returns the reference of this array.
    #[inline]
    pub fn ref_2d<'a>(&self) -> &[&'a [T]] {
        unsafe { slice::from_raw_parts(self.heads.cast().as_ptr(), self.h_size()) }
    }

    /// Returns the mutable slice of all values in the array.
    #[inline]
    pub fn mut_1d(&mut self) -> &mut [T] {
        unsafe {
            let x = self.head_mut();
            let vlen = x.len();
            slice::from_raw_parts_mut(x.as_mut_ptr(), self.h_size() * vlen)
        }
    }

    /// Returns the mutable reference of this array.
    #[inline]
    pub fn mut_2d<'a>(&mut self) -> &mut [&'a mut [T]] {
        unsafe { slice::from_raw_parts_mut(self.heads.cast().as_mut(), self.h_size()) }
    }

    /// Returns the underlying [`Vec`] consuming this [`FixedVec2D`]
    pub fn into_raw(self) -> Vec<T> {
        unsafe { ManuallyDrop::new(self).into_raw_inner(true) }
    }

    ///Dropping the returned vec will drop the values in [`FixedVec2D`].
    ///Be careful not to accidentaly drops the inner values.
    unsafe fn into_raw_inner(&self, drop_heads: bool) -> Vec<T> {
        let hlen = self.h_size();
        let x = self.head_mut();
        let len = hlen * x.len();
        let v_val = Vec::from_raw_parts(x.as_mut_ptr(), len, len);
        if drop_heads {
            let v_head = Vec::from_raw_parts(self.heads.as_ptr(), 0, hlen);
            drop(v_head);
        }
        v_val
    }

    #[inline]
    unsafe fn head_mut(&self) -> &mut [T] {
        self.heads.as_ref().as_mut().unwrap_or_else(|| {
            debug_assert!(false, "heads should not be empty");
            std::hint::unreachable_unchecked()
        })
    }
}

impl<T> FixedVec2D<MaybeUninit<T>> {
    /**
    Assume init. Use this with [`new_uninit`](`Self::new_uninit`).
    ```
    # use std::mem::MaybeUninit;
    # use std::num::NonZeroUsize;
    # use lattice_graph::fixedvec2d::FixedVec2D;
    let mut array = unsafe { FixedVec2D::<MaybeUninit<i32>>::new_uninit(NonZeroUsize::new(6).unwrap(),3) };
    for i in 0..6{
        for j in 0..3{
            array.mut_2d()[i][j] = MaybeUninit::new((i + j) as i32);
        }
    }
    let array_init = unsafe { array.assume_init() };
    ```
    */
    pub unsafe fn assume_init(self) -> FixedVec2D<T> {
        let m = ManuallyDrop::new(self);
        FixedVec2D {
            heads: m.heads.cast(),
            hsize: m.hsize,
        }
    }
}

impl<T: Clone> Clone for FixedVec2D<T> {
    fn clone(&self) -> Self {
        unsafe {
            let vec = self.into_raw_inner(false);
            let vec = ManuallyDrop::new(vec);
            let vec_c = ManuallyDrop::into_inner(vec.clone());
            Self::from_raw_unchecked(self.hsize, self.v_size(), vec_c)
        }
    }
}

impl<T: Debug> Debug for FixedVec2D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.ref_2d().fmt(f)
    }
}

impl<T: PartialEq> PartialEq for FixedVec2D<T> {
    fn eq(&self, other: &Self) -> bool {
        self.h_size() == other.h_size() && self.ref_1d() == other.ref_1d()
    }
}

impl<T: PartialEq> Eq for FixedVec2D<T> {}

impl<T: Hash> Hash for FixedVec2D<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.h_size().hash(state);
        self.ref_1d().hash(state)
    }
}

impl<'a, T> AsRef<[&'a [T]]> for FixedVec2D<T> {
    fn as_ref(&self) -> &[&'a [T]] {
        self.ref_2d()
    }
}

impl<'a, T> AsMut<[&'a mut [T]]> for FixedVec2D<T> {
    fn as_mut(&mut self) -> &mut [&'a mut [T]] {
        self.mut_2d()
    }
}

impl<T> AsRef<[T]> for FixedVec2D<T> {
    fn as_ref(&self) -> &[T] {
        self.ref_1d()
    }
}

impl<T> AsMut<[T]> for FixedVec2D<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.mut_1d()
    }
}

impl<T> Drop for FixedVec2D<T> {
    fn drop(&mut self) {
        drop(unsafe { self.into_raw_inner(true) })
    }
}

impl<T> Index<(usize, usize)> for FixedVec2D<T> {
    type Output = T;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.ref_2d()[index.0][index.1]
    }
}

impl<T> IndexMut<(usize, usize)> for FixedVec2D<T> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.mut_2d()[index.0][index.1]
    }
}

#[cfg(test)]
mod tests {
    type Nz = std::num::NonZeroUsize;
    use std::mem::MaybeUninit;

    use super::FixedVec2D;

    #[test]
    fn gen_0() {
        let x = FixedVec2D::new(Nz::new(5).unwrap(), 0, |h, v| (h, v));
        let ar2d = x.ref_2d();
        assert_eq!(ar2d.len(), 5);
        assert_eq!(ar2d.get(0).map(|x| x.len()), Some(0));
        let ar1d = x.ref_1d();
        assert_eq!(ar1d.len(), 0);
    }

    #[test]
    fn gen() {
        let x = FixedVec2D::new(Nz::new(5).unwrap(), 2, |h, v| (h, v));
        for i in 0..5 {
            for j in 0..2 {
                assert_eq!(x.ref_2d()[i][j], (i, j));
            }
        }
    }

    #[test]
    fn ref_1d() {
        let x = FixedVec2D::new(Nz::new(5).unwrap(), 2, |h, v| (h, v));
        for i in 0..5 {
            for j in 0..2 {
                assert_eq!(x.ref_1d()[j + i * 2], (i, j));
            }
        }
    }

    #[test]
    fn gen_zst() {
        let x = FixedVec2D::new(Nz::new(5).unwrap(), 2, |_h, _v| ());
        for i in 0..5 {
            for j in 0..2 {
                assert_eq!(x.ref_2d()[i][j], ());
            }
        }
    }

    #[test]
    fn clone() {
        let x = FixedVec2D::new(Nz::new(5).unwrap(), 2, |h, v| (h, v));
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

    #[test]
    fn uninit() {
        let mut array =
            unsafe { FixedVec2D::<MaybeUninit<i32>>::new_uninit(Nz::new(6).unwrap(), 3) };
        for i in 0..6 {
            for j in 0..3 {
                array.mut_2d()[i][j] = MaybeUninit::new((i + j) as i32);
            }
        }
        let array_init = unsafe { array.assume_init() };
        for i in 0..6 {
            for j in 0..3 {
                assert_eq!(array_init.ref_2d()[i][j], (i + j) as i32);
            }
        }
    }
}
