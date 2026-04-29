use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Top-level sdi-rust configuration, assembled from the 5-level precedence chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Source-file discovery and global analysis settings.
    pub core: CoreConfig,
    /// Snapshot storage settings.
    pub snapshots: SnapshotConfig,
    /// Boundary detection settings.
    pub boundaries: BoundariesConfig,
    /// Pattern catalog settings.
    pub patterns: PatternsConfig,
    /// Divergence rate thresholds.
    pub thresholds: ThresholdsConfig,
    /// Change-coupling analysis settings.
    pub change_coupling: ChangeCouplingConfig,
    /// Output format and colour settings.
    pub output: OutputConfig,
    /// Determinism enforcement settings (sdi-rust only).
    pub determinism: DeterminismConfig,
    /// Reserved for future binding-specific knobs.
    pub bindings: BindingsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            core: CoreConfig::default(),
            snapshots: SnapshotConfig::default(),
            boundaries: BoundariesConfig::default(),
            patterns: PatternsConfig::default(),
            thresholds: ThresholdsConfig::default(),
            change_coupling: ChangeCouplingConfig::default(),
            output: OutputConfig::default(),
            determinism: DeterminismConfig::default(),
            bindings: BindingsConfig::default(),
        }
    }
}

/// Source-file discovery and analysis settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    /// Languages to analyse. `"auto"` detects from the repository; an explicit
    /// list restricts analysis to the named languages.
    pub languages: String,
    /// Glob patterns for paths to exclude from analysis.
    pub exclude: Vec<String>,
    /// Seed for all deterministic RNG operations in the pipeline.
    pub random_seed: u64,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            languages: "auto".to_string(),
            exclude: vec![
                "**/vendor/**".to_string(),
                "**/node_modules/**".to_string(),
                "**/__pycache__/**".to_string(),
                "**/dist/**".to_string(),
                "**/build/**".to_string(),
                "**/target/**".to_string(),
                "**/.git/**".to_string(),
            ],
            random_seed: 42,
        }
    }
}

/// Snapshot storage settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotConfig {
    /// Directory where snapshots are written (relative to repo root).
    pub dir: String,
    /// Maximum number of snapshots to retain. `0` means unlimited.
    pub retention: u32,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            dir: ".sdi/snapshots".to_string(),
            retention: 100,
        }
    }
}

/// Boundary detection and Leiden community-detection settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundariesConfig {
    /// Path to the boundary specification YAML file (relative to repo root).
    pub spec_file: String,
    /// Leiden resolution parameter (gamma). Manual override only.
    pub leiden_gamma: f64,
    /// Minimum number of consecutive snapshots a community must be stable before
    /// it is proposed as a boundary.
    pub stability_threshold: u32,
    /// Whether to weight graph edges by coupling frequency.
    pub weighted_edges: bool,
}

impl Default for BoundariesConfig {
    fn default() -> Self {
        Self {
            spec_file: ".sdi/boundaries.yaml".to_string(),
            leiden_gamma: 1.0,
            stability_threshold: 3,
            weighted_edges: false,
        }
    }
}

/// Pattern catalog settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternsConfig {
    /// Pattern categories to compute. `"auto"` enables all built-in categories.
    pub categories: String,
    /// Minimum number of nodes a pattern must appear in to be included.
    pub min_pattern_nodes: u32,
    /// Glob patterns for paths excluded from the catalog (files remain in graph).
    pub scope_exclude: Vec<String>,
}

impl Default for PatternsConfig {
    fn default() -> Self {
        Self {
            categories: "auto".to_string(),
            min_pattern_nodes: 5,
            scope_exclude: vec![],
        }
    }
}

/// Divergence rate thresholds for `sdi check`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdsConfig {
    /// Maximum allowed pattern entropy rate (per snapshot interval).
    pub pattern_entropy_rate: f64,
    /// Maximum allowed convention drift rate.
    pub convention_drift_rate: f64,
    /// Maximum allowed coupling delta rate.
    pub coupling_delta_rate: f64,
    /// Maximum allowed boundary violation rate.
    pub boundary_violation_rate: f64,
    /// Per-category threshold overrides with mandatory expiry dates.
    #[serde(default)]
    pub overrides: BTreeMap<String, ThresholdOverride>,
}

impl Default for ThresholdsConfig {
    fn default() -> Self {
        Self {
            pattern_entropy_rate: 2.0,
            convention_drift_rate: 3.0,
            coupling_delta_rate: 0.15,
            boundary_violation_rate: 2.0,
            overrides: BTreeMap::new(),
        }
    }
}

/// A per-category threshold override with a mandatory expiry date.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdOverride {
    /// Overridden pattern entropy rate (inherits default if absent).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern_entropy_rate: Option<f64>,
    /// Overridden convention drift rate (inherits default if absent).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub convention_drift_rate: Option<f64>,
    /// Overridden coupling delta rate (inherits default if absent).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coupling_delta_rate: Option<f64>,
    /// Overridden boundary violation rate (inherits default if absent).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boundary_violation_rate: Option<f64>,
    /// ISO-8601 date after which this override is silently ignored.
    /// **Required** — absence is a `ConfigError::MissingExpiresOnOverride`.
    pub expires: String,
    /// Human-readable explanation for the override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Change-coupling analysis settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeCouplingConfig {
    /// Minimum co-change frequency (0–1) for a pair to be reported.
    pub min_frequency: f64,
    /// Number of commits to inspect for change-coupling analysis.
    pub history_depth: u32,
}

impl Default for ChangeCouplingConfig {
    fn default() -> Self {
        Self {
            min_frequency: 0.6,
            history_depth: 500,
        }
    }
}

/// Output format for `sdi` commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Human-readable table/text output (default).
    #[default]
    Text,
    /// Machine-readable JSON on stdout.
    Json,
}

/// ANSI colour output mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ColorChoice {
    /// Enable colour when the terminal supports it (default).
    #[default]
    Auto,
    /// Always emit ANSI colour codes.
    Always,
    /// Never emit ANSI colour codes (also set by `NO_COLOR=1`).
    Never,
}

/// Output format and colour settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Output format for human-readable commands.
    pub format: OutputFormat,
    /// ANSI colour output mode.
    pub color: ColorChoice,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: OutputFormat::Text,
            color: ColorChoice::Auto,
        }
    }
}

/// Determinism enforcement settings (sdi-rust only; reserved for FMA toggles).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterminismConfig {
    /// Enforce `BTreeMap` ordering throughout the pipeline output.
    pub enforce_btree_order: bool,
}

impl Default for DeterminismConfig {
    fn default() -> Self {
        Self {
            enforce_btree_order: true,
        }
    }
}

/// Reserved for future binding-specific knobs (post-MVP).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BindingsConfig {}
