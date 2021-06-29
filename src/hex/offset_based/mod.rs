use crate::lattice_abstract::LatticeGraph;
use shapes::*;
pub mod shapes;

pub type HexGraphOffsetOddR<N, E, H = usize, V = usize> =
    LatticeGraph<N, E, HexOffsetShape<OddR, H, V>>;

#[cfg(feature = "const-generic-wrap")]
pub type HexGraphOffsetConstOddR<N, E, const H: usize, const V: usize> =
    LatticeGraph<N, E, super::offset_based::ConstHexOffsetShape<OddR, H, V>>;
