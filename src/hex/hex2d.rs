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
pub type HexShapeConst<B, L, I, const H: usize, const V: usize> =
    HexShape<B, L, I, WrapUSIZE<H>, WrapUSIZE<V>>;
impl<B, L, I: hex2d::Integer, const H: usize, const V: usize> Default
    for HexShapeConst<B, L, I, H, V>
{
    fn default() -> Self {
        Self::new(WrapUSIZE::<H>, WrapUSIZE::<V>)
    }
}

#[cfg(feature = "const-generic-wrap")]
/// Hex Graph with [`Coordinate`](`hex2d::Coordinate`) as ZST.
pub type HexGraphConst<N, E, B, L, I, const H: usize, const V: usize> =
    LatticeGraph<N, E, HexShapeConst<B, L, I, H, V>>;

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

#[cfg(test)]
mod tests {
    type C = hex2d::Coordinate;
    use super::*;
    use crate::hex::shapes::{AxisR, OddR};
    use petgraph::{data::DataMap, visit::*};
    use rstest::*;
    type Hex5x5 = HexGraphConst<C, (C, AxisR), OddR, (), i32, 5, 5>;

    #[fixture]
    fn hexgraph_oddr55() -> Hex5x5 {
        Hex5x5::new_with_s(|x| (x), |n, d| (n, d))
    }
    #[rstest]
    fn gen_oddr(hexgraph_oddr55: Hex5x5) {
        let graph = hexgraph_oddr55;
        for i in 0..graph.node_count() {
            let x = graph.from_index(i);
            assert_eq!(Some(&x), graph.node_weight(x));
        }
        assert_eq!(0, std::mem::size_of_val(graph.shape()))
    }

    #[rstest]
    #[case(C::new(0, 0),[C::new(0, 1), C::new(1, 0)] )]
    #[case(C::new(4, 0),[C::new(4, 1), C::new(3, 0), C::new(3, 1)] )]
    #[case(C::new(1, 1),[
        C::new(1, 2),
        C::new(2, 1),
        C::new(2, 0),
        C::new(1, 0),
        C::new(0, 1),
        C::new(0, 2),
    ]) ]
    #[case(C::new(1, 2),[
        C::new(1, 3),
        C::new(2, 2),
        C::new(2, 1),
        C::new(1, 1),
        C::new(0, 2),
        C::new(0, 3),
    ]) ]
    fn neighbors_oddr(
        hexgraph_oddr55: Hex5x5,
        #[case] target: C,
        #[case] neighbors: impl IntoIterator<Item = C>,
    ) {
        let graph = hexgraph_oddr55;
        let e = graph.neighbors(target);
        debug_assert!(e.eq(neighbors));
        let e = graph.neighbors(target);
        for neighbor in e {
            assert_eq!(neighbor.distance(target), 1);
        }
    }
}
