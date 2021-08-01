/*!
Module for Hex Graph with axial coordinates.
It doesn't have a Even Offset coloumn/rows right now.
*/

mod shapes;
use super::shapes::OddR;
use crate::lattice_abstract::LatticeGraph;
pub use shapes::{ConstDoubleCoordShape, DoubleCoord, DoubleCoordShape};
///Hex Graph with double coordinates.
pub type HexGraph<N, E, B = OddR, L = (), H = usize, V = usize> =
    LatticeGraph<N, E, DoubleCoordShape<B, L, H, V>>;
///Hex Graph with double coordinates. The size is const fixed.
#[cfg(feature = "const-generic-wrap")]
pub type HexGraphConst<N, E, B, const H: usize, const V: usize> =
    LatticeGraph<N, E, ConstDoubleCoordShape<B, (), H, V>>;
