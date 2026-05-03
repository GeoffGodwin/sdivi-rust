//! Pure-compute functions — all referentially transparent, WASM-compatible.

pub mod boundaries;
pub mod change_coupling;
pub mod coupling;
pub mod normalize;
pub mod patterns;
pub mod thresholds;
mod stability;
mod violation;
