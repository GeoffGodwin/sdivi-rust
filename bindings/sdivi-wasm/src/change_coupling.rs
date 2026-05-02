//! WASM types and export for compute_change_coupling.

use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

fn err(e: impl std::fmt::Display) -> JsError {
    JsError::new(&e.to_string())
}

fn to_core<W: Serialize, C: for<'de> Deserialize<'de>>(w: W) -> Result<C, JsError> {
    let v = serde_json::to_value(w).map_err(err)?;
    serde_json::from_value(v).map_err(err)
}

fn from_core<C: Serialize, W: for<'de> Deserialize<'de>>(c: C) -> Result<W, JsError> {
    let v = serde_json::to_value(c).map_err(err)?;
    serde_json::from_value(v).map_err(err)
}

/// A single commit event for change-coupling analysis.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmCoChangeEventInput {
    /// Git commit SHA (hex string).
    pub commit_sha: String,
    /// ISO-8601 UTC commit date.
    pub commit_date: String,
    /// Canonical NodeIds of files touched by this commit.
    pub files: Vec<String>,
}

/// Configuration for compute_change_coupling.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmChangeCouplingConfigInput {
    /// Minimum co-change frequency (0.0–1.0).
    pub min_frequency: f64,
    /// Maximum number of commits to analyze.
    pub history_depth: u32,
}

/// A file pair that co-changes above min_frequency.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmCoChangePair {
    /// Lexicographically smaller file path.
    pub source: String,
    /// Lexicographically larger file path.
    pub target: String,
    /// Co-change frequency: cochange_count / commits_analyzed.
    pub frequency: f64,
    /// Number of commits that touched both files.
    pub cochange_count: u32,
}

/// Result of compute_change_coupling.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmChangeCouplingResult {
    /// Sorted file pairs meeting min_frequency with cochange_count >= 2.
    pub pairs: Vec<WasmCoChangePair>,
    /// Number of commits actually analyzed.
    pub commits_analyzed: u32,
    /// Count of unique file paths in analyzed commits.
    pub distinct_files_touched: u32,
}

/// Compute file-pair co-change frequencies from a list of commit events.
///
/// Pure function — no I/O, no clock. Suitable for Meridian and other
/// consumers that supply their own commit-history extractor.
#[wasm_bindgen]
pub fn compute_change_coupling(
    events: Vec<WasmCoChangeEventInput>,
    cfg: WasmChangeCouplingConfigInput,
) -> Result<WasmChangeCouplingResult, JsError> {
    let e: Vec<sdivi_core::CoChangeEventInput> =
        events.into_iter().map(to_core).collect::<Result<_, _>>()?;
    let c = to_core(cfg)?;
    let result = sdivi_core::compute_change_coupling(&e, &c).map_err(err)?;
    from_core(result)
}
