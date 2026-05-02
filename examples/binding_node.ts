/**
 * binding_node.ts — consumer-app-shaped usage of @geoffgodwin/sdivi-wasm.
 *
 * Demonstrates the init-then-call pattern.  In real use the caller supplies
 * extracted graph/pattern data; here we use a small inline fixture.
 *
 * Run (after `wasm-pack build --target bundler --release` in bindings/sdivi-wasm):
 *   npx tsx examples/binding_node.ts
 */
import init, {
  detect_boundaries,
  compute_coupling_topology,
  compute_pattern_metrics,
  compute_delta,
  assemble_snapshot,
  normalize_and_hash,
} from '@geoffgodwin/sdivi-wasm';

// Types are available from the generated .d.ts — no manual interface needed.

async function main() {
  // 1. Initialise the WASM module (required once before any compute calls).
  await init();

  // 2. Build a small dependency graph.
  const graph = {
    nodes: [
      { id: 'src/lib.rs',    path: 'src/lib.rs',    language: 'rust' },
      { id: 'src/models.rs', path: 'src/models.rs', language: 'rust' },
      { id: 'src/config.rs', path: 'src/config.rs', language: 'rust' },
    ],
    edges: [
      { source: 'src/lib.rs',    target: 'src/models.rs' },
      { source: 'src/lib.rs',    target: 'src/config.rs' },
      { source: 'src/models.rs', target: 'src/config.rs' },
    ],
  };

  // 3. Compute coupling metrics.
  const coupling = compute_coupling_topology(graph);
  console.log('graph density:', coupling.density.toFixed(4));
  console.log('top hubs:', coupling.top_hubs);

  // 4. Detect communities / boundaries.
  const cfg = { seed: 42, gamma: 1.0, iterations: 100, quality: 'Modularity' as const };
  const boundaries = detect_boundaries(graph, cfg, []);
  console.log('communities detected:', boundaries.community_count);
  console.log('cluster assignments:', boundaries.cluster_assignments);

  // 5. Compute pattern metrics from extracted instances.
  const patterns = [
    { fingerprint: 'a'.repeat(64), category: 'error_handling', node_id: 'src/lib.rs' },
    { fingerprint: 'b'.repeat(64), category: 'error_handling', node_id: 'src/models.rs' },
    { fingerprint: 'a'.repeat(64), category: 'error_handling', node_id: 'src/config.rs' },
  ];
  const pm = compute_pattern_metrics(patterns);
  console.log('total_entropy:', pm.total_entropy.toFixed(4));
  console.log('convention_drift:', pm.convention_drift.toFixed(4));

  // 6. normalize_and_hash — produces the same blake3 digest as the native Rust pipeline.
  const hash = normalize_and_hash('try_expression', []);
  console.log('normalize_and_hash("try_expression", []):', hash);

  // 7. Assemble a Snapshot from the compute outputs.
  const nodeIds = graph.nodes.map((n) => n.id);
  const snapshot1 = assemble_snapshot({
    node_ids: nodeIds,
    cluster_assignments: boundaries.cluster_assignments,
    internal_edge_density: boundaries.internal_edge_density,
    modularity: boundaries.modularity,
    node_count: coupling.node_count,
    edge_count: coupling.edge_count,
    density: coupling.density,
    cycle_count: coupling.cycle_count,
    top_hubs: coupling.top_hubs,
    component_count: coupling.component_count,
    pattern_metrics: pm,
    pattern_instances: patterns,
    timestamp: new Date().toISOString(),
    commit: undefined,
    boundary_count: undefined,
    leiden_seed: cfg.seed,
  });
  console.log('snapshot_version:', snapshot1.snapshot_version);

  // 8. Compute delta between two snapshots (same snapshot → zero deltas).
  const delta = compute_delta(snapshot1, snapshot1);
  console.log('coupling_delta (same→same):', delta.coupling_delta); // 0
}

main().catch(console.error);
