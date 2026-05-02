//! Configuration loading and validation for sdivi-rust.
//!
//! # Usage
//!
//! ```rust
//! use sdivi_config::Config;
//!
//! let config = Config::default();
//! assert_eq!(config.core.random_seed, 42);
//! ```

mod boundary;
mod config;
mod error;
pub(crate) mod thresholds;

#[cfg(feature = "loader")]
mod load;

pub use boundary::{BoundaryDef, BoundarySpec};
pub use config::{
    BindingsConfig, BoundariesConfig, ChangeCouplingConfig, ColorChoice, Config, CoreConfig,
    DeterminismConfig, OutputConfig, OutputFormat, PatternsConfig, SnapshotConfig,
    ThresholdOverride, ThresholdsConfig,
};
pub use error::ConfigError;
pub use thresholds::validate_date_format;

#[cfg(feature = "loader")]
pub use load::{load_or_default, load_with_paths, project_config_path};
