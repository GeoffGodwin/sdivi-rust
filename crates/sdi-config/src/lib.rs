//! Configuration loading and validation for sdi-rust.
//!
//! # Usage
//!
//! ```rust
//! use sdi_config::Config;
//!
//! let config = Config::default();
//! assert_eq!(config.core.random_seed, 42);
//! ```

mod config;
mod error;
mod load;

pub use config::{
    BindingsConfig, BoundariesConfig, ChangeCouplingConfig, ColorChoice, Config, CoreConfig,
    DeterminismConfig, OutputConfig, OutputFormat, PatternsConfig, SnapshotConfig,
    ThresholdOverride, ThresholdsConfig,
};
pub use error::ConfigError;
pub use load::load_or_default;
