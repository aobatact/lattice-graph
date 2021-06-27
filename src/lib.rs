pub mod fixedvec2d;
pub mod square;
pub use square::SquareGraph;
mod lattice_abstract;

#[inline]
pub(crate) unsafe fn unreachable_debug_checked<T>() -> T {
    if cfg!(debug_assertion) {
        unreachable!()
    } else {
        core::hint::unreachable_unchecked()
    }
}
