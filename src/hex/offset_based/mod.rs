use crate::lattice_abstract::LatticeGraph;
use shapes::*;
pub mod shapes;

pub type HexGraph<N, E, B, H = usize, V = usize> = LatticeGraph<N, E, HexOffsetShape<B, H, V>>;

#[cfg(feature = "const-generic-wrap")]
pub type HexGraphConst<N, E, B, const H: usize, const V: usize> =
    LatticeGraph<N, E, super::offset_based::ConstHexOffsetShape<B, H, V>>;
