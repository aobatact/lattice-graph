/*!
Module to use [`hex2d`] as Coordinate of Hex Graph.

This Module is to use [`hex2d`] as Coordinate of Hex Graph.
The behavior is almost same as [`axial_based`](`super::axial_based`).

Currently this [`HexShape`](`HexShape`) doesn't use [`hex2d::Direction`]
as [`Axis`] or [`AxisDirection`](`crate::lattice_abstract::shapes::AxisDirection`)
to reuse the implemention with [`axial_based`](`super::axial_based`).
So it isn't perfectly integrate with it, so I might make another
`Shape` and `Graph` if I have time.
*/

use super::{
    axial_based::{shapes::AxialCoord, HexAxialShape},
    shapes::{AxisDQ, AxisDR, OddR},
};
use crate::{
    lattice_abstract::{shapes::Axis, LatticeGraph},
    unreachable_debug_checked,
};
#[cfg(feature = "const-generic-wrap")]
use const_generic_wrap::WrapUSIZE;
use hex2d::{Direction, Integer};

impl<I: Integer> crate::lattice_abstract::shapes::Coordinate for hex2d::Coordinate<I> {}
impl<I: Integer> AxialCoord for hex2d::Coordinate<I> {
    fn new(r: isize, q: isize) -> Self {
        Self::new(
            I::from_isize(r).unwrap_or_else(|| unsafe { unreachable_debug_checked() }),
            I::from_isize(q).unwrap_or_else(|| unsafe { unreachable_debug_checked() }),
        )
    }

    fn r(&self) -> isize {
        self.x
            .to_isize()
            .unwrap_or_else(|| unsafe { unreachable_debug_checked() })
    }

    fn q(&self) -> isize {
        self.y
            .to_isize()
            .unwrap_or_else(|| unsafe { unreachable_debug_checked() })
    }
}

impl Axis for Direction {
    const COUNT: usize = 6;
    const DIRECTED: bool = true;
    type Direction = Self;

    fn to_index(&self) -> usize {
        self.to_int::<isize>() as usize
    }

    fn from_index(index: usize) -> Option<Self>
    where
        Self: Sized,
    {
        if index < Self::COUNT {
            Some(Self::from_int(index as isize))
        } else {
            None
        }
    }

    unsafe fn from_index_unchecked(index: usize) -> Self {
        Self::from_int(index as isize)
    }

    fn foward(self) -> Self::Direction {
        self
    }

    fn backward(self) -> Self::Direction {
        let mut x = self.to_index() + 3;
        if x > 6 {
            x -= 6;
        }
        unsafe { Self::from_index_unchecked(x) }
    }

    fn from_direction(dir: Self::Direction) -> Self {
        dir
    }
}

pub struct Hex2dShape;

pub mod axial_based_dir;
