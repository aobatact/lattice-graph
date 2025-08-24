/*!
Extention library for [petgraph](https://crates.io/crates/petgraph).
This adds a specialized graph for lattice (or grid) based graph structures for petgraph.
This probides a smaller and faster graph than the general purpose `petgraph::Graph` struct.
It can be used for path finding in tilemap based game.
This is for developing game, but it can be used for other purposes as well.

# features
## const-generic-wrap
Use [`const-generic-wrap`](`const_generic_wrap`) to make it possible to make some
[`Shape`](`crate::lattice_abstract::shapes::Shape`) to be ZST.

This needs const generics (rustc >= 1.51) to use.
This is enabled by default, so if you want to use this crate with rustc < 1.51,
set default-features as false.

## hex2d
Use [`hex2d`](`hex2d`) as a
[`shapes::Coordinate`](`crate::lattice_abstract::shapes::Coordinate`).
See [`hex::hex2d`] for details.
*/

#![allow(clippy::missing_safety_doc)]

// fixedvec2d module replaced with ndarray
pub use ndarray::{Array2, ArrayView2, ArrayViewMut2};
pub mod square;
pub use square::SquareGraph;
pub mod hex;
pub mod lattice_abstract;

#[inline(always)]
pub(crate) unsafe fn unreachable_debug_checked<T>() -> T {
    if cfg!(debug_assertions) {
        unreachable!()
    } else {
        core::hint::unreachable_unchecked()
    }
}
