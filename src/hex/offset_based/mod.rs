/*!
Module for Hex Graph with offset based coordinates.
Offset coordinate is the coordinate pointing the same offset with underlying [`Offset`](`crate::lattice_abstract::Offset`),
so converting to the [`Offset`](`crate::lattice_abstract::Offset) to get the weight of node or edge is just validating.
Though, this might be costly for moving the coordinate ([`move_coord`](`crate::lattice_abstract::Shape::move_coord`)) than [`axial_based`](`super::axial_based`).
*/

use super::shapes::{DirectedMarker, LoopEW};
use crate::lattice_abstract::LatticeGraph;
mod shapes;
pub use shapes::{ConstHexOffsetShape, HexOffset, HexOffsetShape};

///Hex Graph with offset based coordinates.
pub type HexGraph<N, E, B, H = usize, V = usize> = LatticeGraph<N, E, HexOffsetShape<B, (), H, V>>;
///Hex Graph with offset based coordinates and loop in e-w direction.
pub type HexGraphLoopEW<N, E, B, H = usize, V = usize> =
    LatticeGraph<N, E, HexOffsetShape<B, LoopEW, H, V>>;
///Directed Hex Graph with offset based coordinates.
pub type DiHexGraph<N, E, B, Loop = (), H = usize, V = usize> =
    LatticeGraph<N, E, HexOffsetShape<DirectedMarker<B>, Loop, H, V>>;

///Hex Graph with offset based coordinates.
#[cfg(feature = "const-generic-wrap")]
pub type HexGraphConst<N, E, B, const H: usize, const V: usize> =
    LatticeGraph<N, E, ConstHexOffsetShape<B, (), H, V>>;
///Hex Graph with offset based coordinates and loop in e-w direction.
#[cfg(feature = "const-generic-wrap")]
pub type HexGraphConstLoopEW<N, E, B, const H: usize, const V: usize> =
    LatticeGraph<N, E, ConstHexOffsetShape<B, LoopEW, H, V>>;
///Directed Hex Graph with offset based coordinates.
#[cfg(feature = "const-generic-wrap")]
pub type DiHexGraphConst<N, E, B, const H: usize, const V: usize> =
    LatticeGraph<N, E, ConstHexOffsetShape<DirectedMarker<B>, (), H, V>>;

#[cfg(test)]
#[cfg(feature = "const-generic-wrap")]
mod tests {
    use super::*;
    use crate::hex::shapes::*;
    use petgraph::{data::DataMap, visit::*};
    use rstest::*;
    use std::array::IntoIter;

    type Hex5x5 = HexGraphConst<HexOffset, (HexOffset, AxisR), OddR, 5, 5>;
    type Hex5x5Lew = HexGraphConstLoopEW<HexOffset, (HexOffset, AxisR), OddR, 5, 5>;

    #[fixture]
    fn hexgraph_oddr55() -> Hex5x5 {
        Hex5x5::new_with(HexOffsetShape::default(), |x| (x), |n, d| (n, d))
    }

    #[fixture]
    fn hexgraph_oddr55_lew() -> Hex5x5Lew {
        Hex5x5Lew::new_with(HexOffsetShape::default(), |x| (x), |n, d| (n, d))
    }

    #[rstest]
    fn gen_oddr(hexgraph_oddr55: Hex5x5) {
        let graph = hexgraph_oddr55;
        for i in 0..graph.node_count() {
            let x = graph.from_index(i);
            assert_eq!(Some(&x), graph.node_weight(x));
        }
    }

    #[rstest]
    #[case(HexOffset::new(0, 0), IntoIter::new([HexOffset::new(0, 1), HexOffset::new(1, 0)]) )]
    #[case(HexOffset::new(4, 0), IntoIter::new([HexOffset::new(4, 1), HexOffset::new(3, 0), HexOffset::new(3, 1)]) )]
    #[case(HexOffset::new(1, 1), IntoIter::new([
        HexOffset::new(2, 2),
        HexOffset::new(2, 1),
        HexOffset::new(2, 0),
        HexOffset::new(1, 0),
        HexOffset::new(0, 1),
        HexOffset::new(1, 2),
    ]))]
    #[case(HexOffset::new(1, 2), IntoIter::new([
        HexOffset::new(1, 3),
        HexOffset::new(2, 2),
        HexOffset::new(1, 1),
        HexOffset::new(0, 1),
        HexOffset::new(0, 2),
        HexOffset::new(0, 3),
    ]))]
    fn neighbors_oddr(
        hexgraph_oddr55: Hex5x5,
        #[case] target: HexOffset,
        #[case] neighbors: impl Iterator<Item = HexOffset>,
    ) {
        let graph = hexgraph_oddr55;
        let e = graph.neighbors(target);
        debug_assert!(e.eq(neighbors));
    }

    #[rstest]
    #[case(HexOffset::new(0, 0), IntoIter::new([AxisDR::NE, AxisDR::E,]) )]
    #[case(HexOffset::new(4, 0), IntoIter::new([AxisDR::NE, AxisDR::W, AxisDR::NW]) )]
    #[case(HexOffset::new(1, 1), IntoIter::new([AxisDR::NE, AxisDR::E, AxisDR::SE, AxisDR::SW, AxisDR::W, AxisDR::NW]) )]
    #[case(HexOffset::new(1, 2), IntoIter::new([AxisDR::NE, AxisDR::E, AxisDR::SE, AxisDR::SW, AxisDR::W, AxisDR::NW]) )]
    fn edges_oddr(
        hexgraph_oddr55: Hex5x5,
        #[case] target: HexOffset,
        #[case] edge_targets: impl Iterator<Item = AxisDR>,
    ) {
        let graph = hexgraph_oddr55;
        let e = graph.edges(target).map(|e| e.direction().clone());
        debug_assert!(e.eq(edge_targets));
    }

    #[rstest]
    #[case(HexOffset::new(0, 0), IntoIter::new([
        HexOffset::new(0, 1),
        HexOffset::new(1, 0),
        HexOffset::new(4, 0),
        HexOffset::new(4, 1),]) )]
    #[case(HexOffset::new(4, 0), IntoIter::new([
        HexOffset::new(4, 1),
        HexOffset::new(0, 0),
        HexOffset::new(3, 0),
        HexOffset::new(3, 1)]) )]
    #[case(HexOffset::new(1, 1), IntoIter::new([
        HexOffset::new(2, 2),
        HexOffset::new(2, 1),
        HexOffset::new(2, 0),
        HexOffset::new(1, 0),
        HexOffset::new(0, 1),
        HexOffset::new(1, 2),
    ]))]
    #[case(HexOffset::new(1, 2), IntoIter::new([
        HexOffset::new(1, 3),
        HexOffset::new(2, 2),
        HexOffset::new(1, 1),
        HexOffset::new(0, 1),
        HexOffset::new(0, 2),
        HexOffset::new(0, 3),
    ]))]
    fn neighbors_oddr_lew(
        hexgraph_oddr55_lew: Hex5x5Lew,
        #[case] target: HexOffset,
        #[case] neighbors: impl Iterator<Item = HexOffset>,
    ) {
        let graph = hexgraph_oddr55_lew;
        let e = graph.neighbors(target);
        debug_assert!(e.eq(neighbors));
    }

    #[test]
    fn gen_oddr_di() {
        let graph = DiHexGraphConst::<_, _, OddR, 5, 3>::new_with(
            HexOffsetShape::default(),
            |x| (x),
            |n, d| Some((n, d)),
        );
        for i in 0..graph.node_count() {
            let x = graph.from_index(i);
            assert_eq!(Some(&x), graph.node_weight(x));
        }
    }

    #[test]
    fn edges_oddr_dir() {
        let graph = DiHexGraphConst::<_, _, OddR, 5, 5>::new_with(
            HexOffsetShape::default(),
            |x| (x),
            |n, d| Some((n, d)),
        );
        assert!(graph
            .edges(HexOffset::new(0, 0))
            .map(|e| e.source())
            .all(|x| x == HexOffset::new(0, 0)));
        assert!(graph
            .edges(HexOffset::new(1, 1))
            .map(|e| e.source())
            .all(|x| x == HexOffset::new(1, 1)));
    }
}
