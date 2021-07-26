/*!
Module for Hex Graph with axial coordinates.


## Example
[`HexGraph`]`<N, E,`[`OddR`](`super::shapes::OddR`)`>`
```text
(-1, 1) - (0, 2)        (NW)   NE
    \    /    \            \   /
    (0, 1)  - (1, 1)  (W) -  C -  E
    /    \    /            /   \
(0, 0) - (1, 0)        (SW)     SE
```
[`HexGraph`]`<N, E,`[`EvenR`](`super::shapes::OddR`)`>`
```text
    (-1, 1) - (0, 2)     (NW)  NE
    /    \    /            \   /
(0, 1) - (1, 1)       (W) -  C -  E
    \    /    \            /   \
    (0, 0) - (1, 0)     (SW)    SE
```
[`HexGraph`]`<N, E,`[`OddQ`](`super::shapes::OddQ`)`>`
```text
     (1, 1)                 N
     / |  \           (NW)  |   NE
(0, 1) | (2, 0)          \  |  /
  |  \ |  /  |              C
  |  (1, 0)  |           /  |  \
  |  /    \  |        (SW)  |  SE
(0, 0)   (2, -1)           (S)
```
[`HexGraph`]`<N, E,`[`EvenQ`](`super::shapes::EvenQ`)`>`
```text
(0, 1)   (2, 1)           N
  |  \    /  |      (NW)  |   NE
  |  (1,  1) |         \  |  /
  |  /  |  \ |            C
(0, 0)  | (2, 0)       /  |  \
     \  |  /        (SW)  |  SE
     (1,  0)             (S)
```
*/
mod shapes;
pub use super::shapes::*;
pub use crate::lattice_abstract::shapes::*;
use crate::lattice_abstract::LatticeGraph;
pub use shapes::{ConstHexAxialShape, HexAxial, HexAxialShape};
/// Coordinate for Hex Graph with axial coordinates.
pub type Coord = HexAxial;
///Hex Graph with axial coordinates.
pub type HexGraph<N, E, B = OddR, L = (), H = usize, V = usize> =
    LatticeGraph<N, E, HexAxialShape<B, L, H, V>>;
///Hex Graph with axial coordinates.
#[cfg(feature = "const-generic-wrap")]
pub type HexGraphConst<N, E, B, const H: usize, const V: usize> =
    LatticeGraph<N, E, ConstHexAxialShape<B, (), H, V>>;

///Hex Graph with axial coordinates with e-w loop.
pub type HexGraphLoopEW<N, E, B = OddR, H = usize, V = usize> =
    LatticeGraph<N, E, HexAxialShape<B, LoopEW, H, V>>;

///Hex Graph with axial coordinates with e-w loop.
#[cfg(feature = "const-generic-wrap")]
pub type HexGraphConstLoopEW<N, E, B, const H: usize, const V: usize> =
    LatticeGraph<N, E, ConstHexAxialShape<B, LoopEW, H, V>>;
///Directed Hex Graph with axial coordinates.
pub type DiHexGraph<N, E, B = OddR, Loop = (), H = usize, V = usize> =
    LatticeGraph<N, E, HexAxialShape<DirectedMarker<B>, Loop, H, V>>;
///Directed Hex Graph with axial coordinates.
#[cfg(feature = "const-generic-wrap")]
pub type DiHexGraphConst<N, E, B, Loop, const H: usize, const V: usize> =
    LatticeGraph<N, E, ConstHexAxialShape<DirectedMarker<B>, Loop, H, V>>;

#[cfg(test)]
mod tests {
    use std::{array::IntoIter, mem};

    use super::*;
    use crate::hex::shapes::{AxisR, OddR};
    use petgraph::{data::DataMap, visit::*};
    use rstest::*;
    type C = HexAxial;
    type Hex5x5 = HexGraphConst<C, (C, AxisR), OddR, 5, 5>;
    type Hex5x5EQ = HexGraphConst<C, (C, AxisQ), EvenQ, 5, 5>;
    type Hex5x5Lew = HexGraphConstLoopEW<C, (C, AxisR), OddR, 5, 5>;

    #[fixture]
    fn hexgraph_oddr55() -> Hex5x5 {
        Hex5x5::new_with(HexAxialShape::default(), |x| (x), |n, d| Some((n, d)))
    }
    #[fixture]
    fn hexgraph_oddr55_lew() -> Hex5x5Lew {
        Hex5x5Lew::new_with_s(|x| (x), |n, d| Some((n, d)))
    }
    #[fixture]
    fn hexgraph_evenq55() -> Hex5x5EQ {
        Hex5x5EQ::new_with_s(|x| (x), |n, d| Some((n, d)))
    }
    #[rstest]
    fn gen_oddr(hexgraph_oddr55: Hex5x5) {
        let graph = hexgraph_oddr55;
        for i in 0..graph.node_count() {
            let x = graph.from_index(i);
            assert_eq!(Some(&x), graph.node_weight(x));
        }
        assert_eq!(0, mem::size_of_val(graph.shape()))
    }
    #[rstest]
    fn gen_evenq(hexgraph_evenq55: Hex5x5EQ) {
        let graph = hexgraph_evenq55;
        for i in 0..graph.node_count() {
            let x = graph.from_index(i);
            assert_eq!(Some(&x), graph.node_weight(x));
        }
        assert_eq!(0, mem::size_of_val(graph.shape()))
    }

    #[rstest]
    #[case(C::new(0, 0),IntoIter::new([C::new(0, 1), C::new(1, 0)]) )]
    #[case(C::new(4, 0),IntoIter::new([C::new(4, 1), C::new(3, 0), C::new(3, 1)]) )]
    #[case(C::new(1, 1),IntoIter::new([
        C::new(1, 2),
        C::new(2, 1),
        C::new(2, 0),
        C::new(1, 0),
        C::new(0, 1),
        C::new(0, 2),
    ]) )]
    #[case(C::new(1, 2),IntoIter::new([
        C::new(1, 3),
        C::new(2, 2),
        C::new(2, 1),
        C::new(1, 1),
        C::new(0, 2),
        C::new(0, 3),
    ]) )]
    fn neighbors_oddr(
        hexgraph_oddr55: Hex5x5,
        #[case] target: C,
        #[case] neighbors: impl Iterator<Item = C>,
    ) {
        let graph = hexgraph_oddr55;
        let e = graph.neighbors(target);
        debug_assert!(e.eq(neighbors));
    }

    #[rstest]
    #[case(C::new(0, 0), IntoIter::new([
        C::new(0, 1),
        C::new(1, 0),
        C::new(4, 0),
        C::new(4, 1)]) )]
    #[case(C::new(4, 0), IntoIter::new([
        C::new(4, 1),
        C::new(0, 0),
        C::new(3, 0),
        C::new(3, 1)]) )]
    #[case(C::new(1, 1), IntoIter::new([
        C::new(1, 2),
        C::new(2, 1),
        C::new(2, 0),
        C::new(1, 0),
        C::new(0, 1),
        C::new(0, 2),
    ]) )]
    #[case(C::new(1, 2), IntoIter::new([
        C::new(1, 3),
        C::new(2, 2),
        C::new(2, 1),
        C::new(1, 1),
        C::new(0, 2),
        C::new(0, 3),
    ]) )]
    fn neighbors_oddr_lew(
        hexgraph_oddr55_lew: Hex5x5Lew,
        #[case] target: C,
        #[case] neighbors: impl Iterator<Item = C>,
    ) {
        let graph = hexgraph_oddr55_lew;
        let e = graph.neighbors(target);
        debug_assert!(e.eq(neighbors));
    }
}
