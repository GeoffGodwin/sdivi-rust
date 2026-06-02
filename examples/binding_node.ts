/**
 * binding_node.ts — Node.js consumer of @geoffgodwin/sdivi-wasm.
 *
 * On Node, the package's `node` export condition resolves to the wasm-pack
 * nodejs (CJS) target, which loads the `.wasm` synchronously via fs at require
 * time. There is NO init() to call — default-import the module namespace and
 * destructure the functions you need. (This is the same shape the package's
 * CI smoke test in tests/node_smoke/ exercises.)
 *
 * For a webpack/vite/rollup (browser) consumer, see binding_bundler.ts.
 *
 * Run (after `./build.sh` in bindings/sdivi-wasm — builds both targets):
 *   npx tsx examples/binding_node.ts
 */
import sdivi from '@geoffgodwin/sdivi-wasm';

const {
  detect_boundaries,
  compute_coupling_topology,
  compute_pattern_metrics,
  compute_delta,
  assemble_snapshot,
  normalize_and_hash,
} = sdivi;

// Types are available from the generated .d.ts — no manual interface needed.

function main() {
  // 1. Build a small dependency graph (plain arrays of objects — not Maps).
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

  // 2. Compute coupling metrics.
  const coupling = compute_coupling_topology(graph);
  console.log('graph density:', coupling.density.toFixed(4));
  console.log('top hubs:', coupling.top_hubs);

  // 3. Detect communities / boundaries.
  const cfg = { seed: 42, gamma: 1.0, iterations: 100, quality: 'Modularity' as const };
  const boundaries = detect_boundaries(graph, cfg, []);
  console.log('communities detected:', boundaries.community_count);
  // cluster_assignments is a JS Map (node_id -> community_id), not a plain object.
  console.log('cluster assignments:', [...boundaries.cluster_assignments.entries()]);

  // 4. Compute pattern metrics from extracted instances.
  const patterns = [
    { fingerprint: 'a'.repeat(64), category: 'error_handling', node_id: 'src/lib.rs' },
    { fingerprint: 'b'.repeat(64), category: 'error_handling', node_id: 'src/models.rs' },
    { fingerprint: 'a'.repeat(64), category: 'error_handling', node_id: 'src/config.rs' },
  ];
  const pm = compute_pattern_metrics(patterns);
  console.log('total_entropy:', pm.total_entropy.toFixed(4));
  console.log('convention_drift:', pm.convention_drift.toFixed(4));

  // 5. normalize_and_hash — produces the same blake3 digest as the native Rust pipeline.
  const hash = normalize_and_hash('try_expression', []);
  console.log('normalize_and_hash("try_expression", []):', hash);

  // 6. Assemble a Snapshot from the compute outputs. The Map-typed fields
  //    (cluster_assignments, internal_edge_density) are passed straight through
  //    from detect_boundaries — they are already JS Maps. Optional fields
  //    (commit, boundary_count) are simply omitted when not available.
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
    leiden_seed: cfg.seed,
  });
  console.log('snapshot_version:', snapshot1.snapshot_version);

  // 7. Compute delta between two snapshots (same snapshot → zero deltas).
  const delta = compute_delta(snapshot1, snapshot1);
  console.log('coupling_delta (same→same):', delta.coupling_delta); // 0
}

main();
