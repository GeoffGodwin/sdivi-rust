//! File discovery and AST parsing stage for sdi-rust.
//!
//! Provides the [`adapter::LanguageAdapter`] extension point, the
//! [`feature_record::FeatureRecord`] output type, and the
//! [`parse::parse_repository`] entry point.
//!
//! When compiled with the `test-tree-counter` Cargo feature, [`ACTIVE_TREES`]
//! is exposed so language adapters can track live CST objects in tests.

pub mod adapter;
pub mod feature_record;
pub mod parse;
pub mod walker;

#[cfg(feature = "test-tree-counter")]
/// Global counter of live tree-sitter `Tree` objects (test feature only).
///
/// Language adapters increment this on tree creation and decrement it on drop.
/// Tests assert the value returns to zero after each `parse_file` call.
pub static ACTIVE_TREES: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);
