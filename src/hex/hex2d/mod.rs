/*!
Module to use [`hex2d`] as Coordinate of Hex Graph.

This Module is to use [`hex2d`] as Coordinate of Hex Graph.
The behavior is almost same as [`axial_based`](`super::axial_based`).

This [`HexShape`](`HexShape`) doesn't use [`hex2d::Direction`]
as [`Axis`] or [`AxisDirection`](`crate::lattice_abstract::shapes::AxisDirection`)
to reuse the implemention with [`axial_based`](`super::axial_based`) and .
*/

use std::marker::PhantomData;

use super::{
    axial_based::{shapes::AxialCoord, HexAxialShape},
    shapes::{AxisDQ, AxisDR, OddR},
};
use crate::{
    lattice_abstract::{shapes::Axis, LatticeGraph, Shape},
    unreachable_debug_checked,
};
#[cfg(feature = "const-generic-wrap")]
use const_generic_wrap::WrapUSIZE;
use hex2d::{Coordinate, Direction, Integer};

/**
This [`HexShape`](`HexShape`) doesn't use [`hex2d::Direction`]
as [`Axis`] or [`AxisDirection`](`crate::lattice_abstract::shapes::AxisDirection`), instead uses a one of [`axial_based`](`super::axial_based`).
*/
pub mod axial_based_dir;

impl<I: Integer> crate::lattice_abstract::shapes::Coordinate for hex2d::Coordinate<I> {}

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

#[derive(Debug, Clone, Copy)]
pub struct Hex2dShape<Base, I = i32, H = usize, V = usize> {
    h: H,
    v: V,
    ipd: PhantomData<I>,
    bpd: PhantomData<Base>,
}

impl<B, I, H, V> Hex2dShape<B, I, H, V> {
    pub fn new(h: H, v: V) -> Self {
        Self {
            h,
            v,
            ipd: PhantomData,
            bpd: PhantomData,
        }
    }
}

impl<B, I, H, V> Shape for Hex2dShape<B, I, H, V>
where
    H: Into<usize> + Clone,
    V: Into<usize> + Clone,
    I: Integer,
    B: Clone,
{
    type Axis = Direction;
    type Coordinate = Coordinate<I>;
    type OffsetConvertError = ();
    type CoordinateMoveError = ();

    fn horizontal(&self) -> usize {
        self.h.clone().into()
    }

    fn vertical(&self) -> usize {
        self.v.clone().into()
    }

    fn to_offset(
        &self,
        coord: Self::Coordinate,
    ) -> Result<crate::lattice_abstract::Offset, Self::OffsetConvertError> {
        todo!()
    }

    fn from_offset(&self, offset: crate::lattice_abstract::Offset) -> Self::Coordinate {
        todo!()
    }

    fn move_coord(
        &self,
        coord: Self::Coordinate,
        dir: Direction,
    ) -> Result<Self::Coordinate, Self::CoordinateMoveError> {
        todo!()
    }
}
