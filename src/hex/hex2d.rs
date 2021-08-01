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
use hex2d::Integer;

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

/// Shape for [`Coordinate`](`hex2d::Coordinate`)
pub type HexShape<B, L, I = i32, H = usize, V = usize> =
    HexAxialShape<B, L, H, V, hex2d::Coordinate<I>>;
/// Hex Graph with [`Coordinate`](`hex2d::Coordinate`).
pub type HexGraph<N, E, B = OddR, L = (), I = i32, H = usize, V = usize> =
    LatticeGraph<N, E, HexShape<B, L, I, H, V>>;

#[cfg(feature = "const-generic-wrap")]
/// Shape for [`Coordinate`](`hex2d::Coordinate`) as ZST.
pub type ConstHexShape<B, L, I, const H: usize, const V: usize> =
    HexShape<B, L, I, WrapUSIZE<H>, WrapUSIZE<V>>;
#[cfg(feature = "const-generic-wrap")]
/// Hex Graph with [`Coordinate`](`hex2d::Coordinate`) as ZST.
pub type ConstHexGraph<N, E, B, L, I, const H: usize, const V: usize> =
    LatticeGraph<N, E, ConstHexShape<B, L, I, H, V>>;

impl From<hex2d::Direction> for AxisDR {
    fn from(d: hex2d::Direction) -> Self {
        let i = d.to_int::<isize>() as usize;
        let i = if i == 5 { 0 } else { i + 1 };
        debug_assert!(i <= 5);
        unsafe { Self::from_index_unchecked(i) }
        // match d {
        //     hex2d::Direction::YZ => AxisDR::NW,
        //     hex2d::Direction::XZ => AxisDR::NE,
        //     hex2d::Direction::XY => AxisDR::E,
        //     hex2d::Direction::ZY => AxisDR::SE,
        //     hex2d::Direction::ZX => AxisDR::SW,
        //     hex2d::Direction::YX => AxisDR::W,
        // }
    }
}

impl From<hex2d::Direction> for AxisDQ {
    fn from(d: hex2d::Direction) -> Self {
        let i = d.to_int::<isize>() as usize;
        debug_assert!(i <= 5);
        unsafe { Self::from_index_unchecked(i) }
    }
}

impl From<AxisDR> for hex2d::Direction {
    fn from(d: AxisDR) -> Self {
        let i = d.to_index();
        let i = if i == 0 { 5 } else { i - 1 } as isize;
        debug_assert!(i <= 5);
        Self::from_int(i)
    }
}

impl From<AxisDQ> for hex2d::Direction {
    fn from(d: AxisDQ) -> Self {
        let i = d.to_index() as isize;
        Self::from_int(i)
    }
}
