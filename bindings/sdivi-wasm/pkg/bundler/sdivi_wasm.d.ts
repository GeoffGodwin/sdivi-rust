/* tslint:disable */
/* eslint-disable */

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


/**
 * A directed edge between two nodes.
 */
export interface WasmEdgeInput {
    source: string;
    target: string;
}

/**
 * A file pair that co-changes above min_frequency.
 */
export interface WasmCoChangePair {
    /**
     * Lexicographically smaller file path.
     */
    source: string;
    /**
     * Lexicographically larger file path.
     */
    target: string;
    /**
     * Co-change frequency: cochange_count / commits_analyzed.
     */
    frequency: number;
    /**
     * Number of commits that touched both files.
     */
    cochange_count: number;
}

/**
 * A node in the pattern AST subtree for [`normalize_and_hash`].
 */
export interface WasmNormalizeNode {
    kind: string;
    children: WasmNormalizeNode[];
}

/**
 * A prior partition for [`infer_boundaries`] — mirrors [`sdivi_core::SnapshotPriorPartition`].
 * Kept separate from [`WasmPriorPartition`] to surface struct-shape divergence at compile time.
 */
export interface WasmSnapshotPriorPartition {
    /**
     * node_id → community_id mapping.
     */
    cluster_assignments: Map<string, number>;
}

/**
 * A prior partition for stability scoring.
 */
export interface WasmPriorPartition {
    /**
     * node_id → community_id mapping.
     */
    cluster_assignments: Map<string, number>;
}

/**
 * A proposed boundary from [`infer_boundaries`].
 */
export interface WasmBoundaryProposal {
    community_id: number;
    stable_snapshots: number;
    node_ids: string[];
}

/**
 * A single boundary definition.
 */
export interface WasmBoundaryDefInput {
    name: string;
    modules: string[];
    allow_imports_from: string[];
}

/**
 * A single commit event for change-coupling analysis.
 */
export interface WasmCoChangeEventInput {
    /**
     * Git commit SHA (hex string).
     */
    commit_sha: string;
    /**
     * ISO-8601 UTC commit date.
     */
    commit_date: string;
    /**
     * Canonical NodeIds of files touched by this commit.
     */
    files: string[];
}

/**
 * A single file-pair entry for [`WasmChangeCouplingInput`].
 *
 * Field names match `sdivi_core::CoChangePair` exactly — the serde round-trip
 * conversion in `assemble_snapshot` is field-name-based.
 */
export interface WasmCoChangePairInput {
    /**
     * The lexicographically smaller file path (`source < target`).
     */
    source: string;
    /**
     * The lexicographically larger file path.
     */
    target: string;
    /**
     * Co-change frequency: `cochange_count / commits_analyzed`.
     */
    frequency: number;
    /**
     * Number of commits that touched both files.
     */
    cochange_count: number;
}

/**
 * A single node in a [`WasmDependencyGraphInput`].
 */
export interface WasmNodeInput {
    id: string;
    path: string;
    language: string;
}

/**
 * A single pattern instance for [`compute_pattern_metrics`].
 */
export interface WasmPatternInstanceInput {
    fingerprint: string;
    category: string;
    node_id: string;
    location?: WasmPatternLocationInput;
}

/**
 * A single threshold breach.
 */
export interface WasmThresholdBreachInfo {
    /**
     * Name of the dimension that exceeded its limit.
     */
    dimension: string;
    /**
     * Category name for per-category breaches; absent for aggregate breaches.
     */
    category?: string;
    /**
     * Observed delta value.
     */
    actual: number;
    /**
     * The limit that was exceeded.
     */
    limit: number;
}

/**
 * Boundary specification for [`compute_boundary_violations`].
 */
export interface WasmBoundarySpecInput {
    boundaries: WasmBoundaryDefInput[];
}

/**
 * Change-coupling result passed into [`WasmAssembleSnapshotInput::change_coupling`].
 *
 * Mirrors `sdivi_core::ChangeCouplingResult`. Typically the direct output of
 * [`crate::compute_change_coupling`] — pass it straight through without conversion.
 */
export interface WasmChangeCouplingInput {
    /**
     * File pairs whose co-change frequency meets `min_frequency`.
     */
    pairs: WasmCoChangePairInput[];
    /**
     * Number of commits actually analyzed.
     */
    commits_analyzed: number;
    /**
     * Count of unique file paths across all analyzed commits.
     */
    distinct_files_touched: number;
}

/**
 * Configuration for compute_change_coupling.
 */
export interface WasmChangeCouplingConfigInput {
    /**
     * Minimum co-change frequency (0.0–1.0).
     */
    min_frequency: number;
    /**
     * Maximum number of commits to analyze.
     */
    history_depth: number;
}

/**
 * Diagnostic info for one entry in `WasmThresholdCheckResult::applied_overrides`.
 */
export interface WasmAppliedOverrideInfo {
    /**
     * Whether the override was active (not expired) at evaluation time.
     */
    active: boolean;
    /**
     * Expiry date as `\"YYYY-MM-DD\"`.
     */
    expires: string;
    /**
     * Human-readable explanation when inactive.
     */
    expired_reason?: string;
}

/**
 * Input dependency graph for pure-compute functions.
 */
export interface WasmDependencyGraphInput {
    nodes: WasmNodeInput[];
    edges: WasmEdgeInput[];
}

/**
 * Input to [`crate::assemble_snapshot`].
 *
 * Collects the outputs of the three primary compute functions plus metadata.
 */
export interface WasmAssembleSnapshotInput {
    /**
     * Ordered node IDs (determines numeric partition indices).
     */
    node_ids: string[];
    /**
     * node_id → community_id (from `detect_boundaries`).
     */
    cluster_assignments: Map<string, number>;
    /**
     * community_id (string) → internal density (from `detect_boundaries`).
     */
    internal_edge_density: Map<string, number>;
    /**
     * Modularity score from Leiden run.
     */
    modularity: number;
    /**
     * Total node count.
     */
    node_count: number;
    /**
     * Total edge count.
     */
    edge_count: number;
    /**
     * Graph density.
     */
    density: number;
    /**
     * Cycle count.
     */
    cycle_count: number;
    /**
     * Top hub entries as `[node_id, out_degree]`.
     */
    top_hubs: [string, number][];
    /**
     * Number of weakly-connected components.
     */
    component_count: number;
    /**
     * Pre-computed pattern metrics.
     */
    pattern_metrics: WasmPatternMetricsResult;
    /**
     * Raw pattern instances used to build the catalog (may be empty).
     */
    pattern_instances: WasmPatternInstanceInput[];
    /**
     * ISO-8601 UTC timestamp for the snapshot.
     */
    timestamp: string;
    /**
     * Optional git commit SHA.
     */
    commit?: string;
    /**
     * Number of declared boundaries (sets intent_divergence when Some).
     */
    boundary_count?: number;
    /**
     * Seed used for the Leiden run that produced `cluster_assignments`.
     * Defaults to 42 when absent (matches `LeidenConfigInput` default).
     */
    leiden_seed?: number;
    /**
     * Number of boundary violations (from `compute_boundary_violations`).
     * When `Some`, sets `intent_divergence.violation_count` in the snapshot.
     */
    violation_count?: number;
    /**
     * Change-coupling result (from [`crate::compute_change_coupling`]).
     * When `Some`, populates the `change_coupling` field of the assembled snapshot,
     * identical to what `sdivi-pipeline` produces for native callers.
     */
    change_coupling?: WasmChangeCouplingInput;
}

/**
 * Leiden algorithm configuration.
 */
export interface WasmLeidenConfigInput {
    seed: number;
    gamma: number;
    iterations: number;
    quality: WasmQualityFunction;
    /**
     * Per-edge weights keyed `\"source:target\"` (first colon splits source/target). `None` = unweighted.
     * Weights must be `>= 0.0` and finite. Edges absent from the graph are silently ignored.
     */
    edge_weights?: Map<string, number>;
}

/**
 * Metadata for a single canonical pattern category — WASM wrapper.
 */
export interface WasmCategoryInfo {
    /**
     * Canonical category name.
     */
    name: string;
    /**
     * Human-readable description of the code constructs this category covers.
     */
    description: string;
}

/**
 * Output of [`compute_boundary_violations`].
 */
export interface WasmBoundaryViolationResult {
    violation_count: number;
    violations: [string, string][];
}

/**
 * Output of [`compute_coupling_topology`].
 */
export interface WasmCouplingTopologyResult {
    node_count: number;
    edge_count: number;
    density: number;
    cycle_count: number;
    top_hubs: [string, number][];
    component_count: number;
}

/**
 * Output of [`compute_thresholds_check`].
 */
export interface WasmThresholdCheckResult {
    /**
     * `true` when at least one threshold was exceeded.
     */
    breached: boolean;
    /**
     * Per-dimension details for each exceeded threshold.
     */
    breaches: WasmThresholdBreachInfo[];
    /**
     * Diagnostic map of every override entry with `active` flag and `expires` date.
     */
    applied_overrides: Map<string, WasmAppliedOverrideInfo>;
}

/**
 * Output of [`compute_trend`].
 */
export interface WasmTrendResult {
    snapshot_count: number;
    pattern_entropy_slope?: number;
    convention_drift_slope?: number;
    coupling_slope?: number;
    community_count_slope?: number;
}

/**
 * Output of [`detect_boundaries`].
 */
export interface WasmBoundaryDetectionResult {
    cluster_assignments: Map<string, number>;
    community_count: number;
    modularity: number;
    /**
     * community_id (as string) → internal edge density.
     */
    internal_edge_density: Map<string, number>;
    historical_stability: number;
    disconnected_components: number;
}

/**
 * Output of [`infer_boundaries`].
 */
export interface WasmBoundaryInferenceResult {
    proposals: WasmBoundaryProposal[];
    partition_count: number;
}

/**
 * Pattern metrics output (also used as a snapshot sub-field).
 */
export interface WasmPatternMetricsResult {
    /**
     * Shannon entropy per category.
     */
    entropy_per_category: Map<string, number>;
    /**
     * Sum of per-category entropies.
     */
    total_entropy: number;
    /**
     * Average `distinct / total` across all categories.
     */
    convention_drift: number;
    /**
     * Per-category `distinct / total` before averaging.
     */
    convention_drift_per_category?: Map<string, number>;
}

/**
 * Per-category threshold override.
 */
export interface WasmThresholdOverrideInput {
    pattern_entropy_rate?: number;
    convention_drift_rate?: number;
    coupling_delta_rate?: number;
    boundary_violation_rate?: number;
    /**
     * ISO-8601 expiry date `\"YYYY-MM-DD\"`.
     */
    expires: string;
}

/**
 * Per-dimension divergence between two snapshots — output of [`compute_delta`].
 */
export interface WasmDivergenceSummary {
    pattern_entropy_delta?: number;
    convention_drift_delta?: number;
    coupling_delta?: number;
    community_count_delta?: number;
    boundary_violation_delta?: number;
    /**
     * Per-category entropy delta; `None` on the first-snapshot path.
     */
    pattern_entropy_per_category_delta?: Map<string, number>;
    /**
     * Per-category convention-drift delta; `None` on the first-snapshot path.
     */
    convention_drift_per_category_delta?: Map<string, number>;
}

/**
 * Quality function for Leiden community detection.
 */
export type WasmQualityFunction = "Modularity" | "Cpm";

/**
 * Result of compute_change_coupling.
 */
export interface WasmChangeCouplingResult {
    /**
     * Sorted file pairs meeting min_frequency with cochange_count >= 2.
     */
    pairs: WasmCoChangePair[];
    /**
     * Number of commits actually analyzed.
     */
    commits_analyzed: number;
    /**
     * Count of unique file paths in analyzed commits.
     */
    distinct_files_touched: number;
}

/**
 * Runtime representation of the canonical pattern-category contract — WASM wrapper.
 *
 * Returned by [`list_categories`](crate::exports::list_categories).
 */
export interface WasmCategoryCatalog {
    /**
     * The `snapshot_version` string this contract applies to (`\"1.0\"`).
     */
    schema_version: string;
    /**
     * All canonical categories in alphabetical order.
     */
    categories: WasmCategoryInfo[];
}

/**
 * Source location of a pattern instance.
 */
export interface WasmPatternLocationInput {
    file: string;
    start_row: number;
    start_col: number;
}

/**
 * Threshold configuration for [`compute_thresholds_check`].
 * `today` is an ISO-8601 date string `\"YYYY-MM-DD\"`.
 */
export interface WasmThresholdsInput {
    pattern_entropy_rate: number;
    convention_drift_rate: number;
    coupling_delta_rate: number;
    boundary_violation_rate: number;
    overrides?: Map<string, WasmThresholdOverrideInput>;
    /**
     * ISO-8601 date for threshold expiry evaluation (e.g. `\"2026-05-01\"`).
     */
    today: string;
}


/**
 * Assemble a Snapshot from compute-function outputs.
 *
 * Returns a snapshot JSON object that can be passed to `compute_delta` or
 * stored in `.sdivi/snapshots/`.
 */
export function assemble_snapshot(input: WasmAssembleSnapshotInput): any;

/**
 * Detect cross-boundary dependency violations against a boundary spec.
 */
export function compute_boundary_violations(graph: WasmDependencyGraphInput, spec: WasmBoundarySpecInput): WasmBoundaryViolationResult;

/**
 * Compute file-pair co-change frequencies from a list of commit events.
 *
 * Pure function — no I/O, no clock. Suitable for consumer-app and other
 * consumers that supply their own commit-history extractor.
 */
export function compute_change_coupling(events: WasmCoChangeEventInput[], cfg: WasmChangeCouplingConfigInput): WasmChangeCouplingResult;

/**
 * Compute dependency graph coupling metrics.
 */
export function compute_coupling_topology(graph: WasmDependencyGraphInput): WasmCouplingTopologyResult;

/**
 * Compute per-dimension divergence between two snapshots (JSON objects).
 */
export function compute_delta(prev: any, curr: any): WasmDivergenceSummary;

/**
 * Compute Shannon entropy and convention drift from pattern instances.
 */
export function compute_pattern_metrics(patterns: WasmPatternInstanceInput[]): WasmPatternMetricsResult;

/**
 * Check whether any dimension of a divergence summary exceeds thresholds.
 */
export function compute_thresholds_check(summary: WasmDivergenceSummary, cfg: WasmThresholdsInput): WasmThresholdCheckResult;

/**
 * Compute trend statistics over an array of snapshot JSON objects.
 */
export function compute_trend(snapshots: any, last_n?: number | null): WasmTrendResult;

/**
 * Run Leiden community detection and return cluster assignments + stability.
 *
 * When `cfg.edge_weights` is `Some`, runs weighted Leiden. Keys must be
 * `"source:target"` strings (first colon separates source from target, so
 * node IDs that themselves contain colons are fully supported). Weights must
 * be `>= 0.0` and finite; edges absent from the graph are silently ignored.
 */
export function detect_boundaries(graph: WasmDependencyGraphInput, cfg: WasmLeidenConfigInput, prior: WasmPriorPartition[]): WasmBoundaryDetectionResult;

/**
 * Infer boundary proposals from a sequence of prior partitions.
 */
export function infer_boundaries(prior_partitions: WasmSnapshotPriorPartition[], stability_threshold: number): WasmBoundaryInferenceResult;

/**
 * Initialise WASM — installs the console_error_panic_hook so that Rust
 * panics surface as readable JS errors in dev builds.
 */
export function init_wasm(): void;

/**
 * Return the canonical pattern-category contract for `snapshot_version "1.0"`.
 *
 * Embedders that supply their own tree-sitter extractors should call this
 * function to discover which category names are valid instead of hard-coding them.
 */
export function list_categories(): WasmCategoryCatalog;

/**
 * Compute a canonical blake3 fingerprint for a pattern AST node.
 *
 * Returns a 64-character lowercase hex string that is byte-identical to the
 * fingerprint produced by the native Rust pipeline for the same input.
 */
export function normalize_and_hash(node_kind: string, children: WasmNormalizeNode[]): string;
