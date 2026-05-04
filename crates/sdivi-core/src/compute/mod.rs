//! Pure-compute functions — all referentially transparent, WASM-compatible.

pub mod boundaries;
pub mod change_coupling;
pub mod coupling;
pub mod normalize;
pub mod patterns;
mod stability;
// Private module: types are re-exported publicly via `pub use super::threshold_types::*` in `thresholds.rs`.
mod threshold_types;
pub mod thresholds;
mod violation;
