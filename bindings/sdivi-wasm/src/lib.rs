//! WASM bindings for sdivi-core — `@geoffgodwin/sdivi-wasm`.
//!
//! All pure-compute functions from `sdivi-core` are exported here with
//! `wasm-bindgen` + `tsify`-derived TypeScript types.
//!
//! ## Usage
//! ```ts
//! import init, { detect_boundaries, normalize_and_hash } from '@geoffgodwin/sdivi-wasm';
//! await init();
//! const hash = normalize_and_hash('try_expression', []);
//! ```

pub mod assemble_types;
pub mod category_types;
pub mod change_coupling;
mod exports;
pub mod threshold_types;
pub mod types;
pub(crate) mod weight_keys;

pub use change_coupling::*;
pub use exports::*;
pub use threshold_types::{
    WasmAppliedOverrideInfo, WasmThresholdBreachInfo, WasmThresholdCheckResult,
    WasmThresholdOverrideInput, WasmThresholdsInput,
};

/// Initialise WASM — installs the console_error_panic_hook so that Rust
/// panics surface as readable JS errors in dev builds.
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn init_wasm() {
    console_error_panic_hook::set_once();
}

// ── TypeScript custom section — Snapshot interface ─────────────────────────
// Snapshot is a complex type whose inner fields come from sdivi-detection,
// sdivi-graph, and sdivi-patterns.  We define its TypeScript interface here rather
// than creating wrapper types for every sub-struct.
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const SNAPSHOT_TS: &str = r#"
/** Versioned snapshot produced by assemble_snapshot or loaded from .sdivi/snapshots/. */
export interface Snapshot {
  snapshot_version: string;
  timestamp: string;
  commit?: string;
  graph: GraphMetrics;
  partition: LeidenPartition;
  catalog: PatternCatalog;
  pattern_metrics: WasmPatternMetricsResult;
  intent_divergence?: IntentDivergenceInfo;
  path_partition?: Record<string, number>;
  change_coupling?: ChangeCouplingResult;
}
export interface GraphMetrics {
  node_count: number;
  edge_count: number;
  density: number;
  cycle_count: number;
  top_hubs: [string, number][];
  component_count: number;
}
export interface LeidenPartition {
  assignments: Record<string, number>;
  stability: Record<string, number>;
  modularity: number;
  /** NOTE: Rust source is u64; JS number cannot exactly represent values above 2^53.
      Default seed 42 is safe. Custom seeds must be <= Number.MAX_SAFE_INTEGER. */
  seed: number;
}
export type PatternCatalog = { entries: Record<string, Record<string, PatternStats>> };
export interface PatternStats { count: number; locations: PatternLocation[]; }
export interface PatternLocation { file: string; start_row: number; start_col: number; }
export interface IntentDivergenceInfo { boundary_count: number; violation_count: number; }
export interface ChangeCouplingResult {
  pairs: CoChangePair[];
  commits_analyzed: number;
  distinct_files_touched: number;
}
export interface CoChangePair {
  source: string;
  target: string;
  frequency: number;
  cochange_count: number;
}
"#;
