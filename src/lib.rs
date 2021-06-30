/*!
Extention library for [petgraph](https://crates.io/crates/petgraph).
This adds a specialized graph for lattice (or grid) based graph structures for petgraph.
This probides a smaller and faster graph than the general purpose `petgraph::Graph` struct.
It can be used for passfinding in tilemap based game.
This is for developing game, but it can be used for other purposes as well.
*/

pub mod fixedvec2d;
pub mod square;
pub use square::SquareGraph;
pub mod hex;
pub mod lattice_abstract;

#[inline]
pub(crate) unsafe fn unreachable_debug_checked<T>() -> T {
    if cfg!(debug_assertion) {
        unreachable!()
    } else {
        core::hint::unreachable_unchecked()
    }
}
