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
     (1, 1)
     / |  \
(0, 1) | (2, 0)       (NW)  N   NE
  |  \ |  /  |           \  |  /
  |  (1, 0)  |              C
  |  /    \  |           /  |  \
(0, 0)   (2, -1)      (SW) (S)  SE
```
[`HexGraph`]`<N, E,`[`EvenQ`](`super::shapes::EvenQ`)`>`
```text
(0, 1)   (2, 1)       (NW)  N   NE
  |  \    /  |           \  |  /
  |  (1,  1) |              C
  |  /  |  \ |           /  |  \
(0, 0)  | (2, 0)      (SW) (S)  SE
     \  |  /
     (1,  0)
```
*/
mod shapes;
use super::shapes::{DirectedMarker, OddR, LEW};
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
    LatticeGraph<N, E, HexAxialShape<B, LEW, H, V>>;

///Hex Graph with axial coordinates with e-w loop.
#[cfg(feature = "const-generic-wrap")]
pub type HexGraphConstLoopEW<N, E, B, const H: usize, const V: usize> =
    LatticeGraph<N, E, ConstHexAxialShape<B, LEW, H, V>>;
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
    use crate::hex::shapes::OddR;
    use petgraph::{data::DataMap, visit::*};
    type C = HexAxial;

    #[test]
    fn gen_oddr() {
        let graph = HexGraphConst::<_, _, OddR, 5, 3>::new_with(
            HexAxialShape::default(),
            |x| (x),
            |n, d| Some((n, d)),
        );
        for i in 0..graph.node_count() {
            let x = graph.from_index(i);
            assert_eq!(Some(&x), graph.node_weight(x));
        }
        assert_eq!(0, mem::size_of_val(graph.shape()))
    }

    #[test]
    fn neighbors_oddr() {
        let graph = HexGraphConst::<_, _, OddR, 5, 5>::new_with(
            HexAxialShape::default(),
            |x| (x),
            |n, d| Some((n, d)),
        );
        let e = graph.neighbors(C::new(0, 0));
        debug_assert!(e.eq(IntoIter::new([C::new(0, 1), C::new(1, 0)])));

        let e = graph.neighbors(C::new(4, 0));
        debug_assert!(e.eq(IntoIter::new([C::new(4, 1), C::new(3, 0), C::new(3, 1)])));

        let e = graph.neighbors(C::new(1, 1));
        debug_assert!(e.eq(IntoIter::new([
            C::new(1, 2),
            C::new(2, 1),
            C::new(2, 0),
            C::new(1, 0),
            C::new(0, 1),
            C::new(0, 2),
        ])));

        let e = graph.neighbors(C::new(1, 2));
        debug_assert!(e.eq(IntoIter::new([
            C::new(1, 3),
            C::new(2, 2),
            C::new(2, 1),
            C::new(1, 1),
            C::new(0, 2),
            C::new(0, 3),
        ])));
    }

    #[test]
    fn neighbors_oddr_lew() {
        let graph = HexGraphConstLoopEW::<_, _, OddR, 5, 5>::new_with(
            HexAxialShape::default(),
            |x| (x),
            |n, d| Some((n, d)),
        );
        let e = graph.neighbors(C::new(0, 0));
        debug_assert!(e.eq(IntoIter::new([
            C::new(0, 1),
            C::new(1, 0),
            C::new(4, 0),
            C::new(4, 1)
        ])));

        let e = graph.neighbors(C::new(4, 0));
        debug_assert!(e.eq(IntoIter::new([
            C::new(4, 1),
            C::new(0, 0),
            C::new(3, 0),
            C::new(3, 1)
        ])));

        let e = graph.neighbors(C::new(1, 1));
        debug_assert!(e.eq(IntoIter::new([
            C::new(1, 2),
            C::new(2, 1),
            C::new(2, 0),
            C::new(1, 0),
            C::new(0, 1),
            C::new(0, 2),
        ])));

        let e = graph.neighbors(C::new(1, 2));
        debug_assert!(e.eq(IntoIter::new([
            C::new(1, 3),
            C::new(2, 2),
            C::new(2, 1),
            C::new(1, 1),
            C::new(0, 2),
            C::new(0, 3),
        ])));
    }
}
