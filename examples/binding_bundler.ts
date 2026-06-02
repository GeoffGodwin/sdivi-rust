/**
 * binding_bundler.ts — webpack/vite/rollup (browser) consumer of
 * @geoffgodwin/sdivi-wasm.
 *
 * The bundler target uses named imports and initialises itself on import: your
 * bundler instantiates the `.wasm` and the `init_wasm` start function runs
 * automatically. There is NO init() to call and NO default export — import the
 * named functions directly and call them.
 *
 * Build this through a bundler (webpack 5+, vite, rollup with @rollup/plugin-wasm).
 * Do NOT point a browser build at the `/node` subpath — it uses require('fs')
 * which bundlers cannot resolve in a browser context. The default import below
 * routes browser/bundler builds to the bundler target automatically.
 */
import {
  detect_boundaries,
  compute_coupling_topology,
  list_categories,
} from '@geoffgodwin/sdivi-wasm';

const graph = {
  nodes: [
    { id: 'src/lib.rs',    path: 'src/lib.rs',    language: 'rust' },
    { id: 'src/models.rs', path: 'src/models.rs', language: 'rust' },
  ],
  edges: [{ source: 'src/lib.rs', target: 'src/models.rs' }],
};

// Category contract — 19 canonical categories at snapshot_version "1.0".
const catalog = list_categories();
console.log(catalog.schema_version, catalog.categories.length); // "1.0" 19

const coupling = compute_coupling_topology(graph);
console.log('density:', coupling.density);

// Weighted Leiden: edge_weights is a JS Map, NOT a plain object. Keys are
// "source:target" strings (first colon splits source/target).
const cfg = {
  seed: 42,
  gamma: 1.0,
  iterations: 100,
  quality: 'Modularity' as const,
  edge_weights: new Map([['src/lib.rs:src/models.rs', 0.8]]),
};
const boundaries = detect_boundaries(graph, cfg, []);

// Outputs are JS Maps too — read with .get()/.entries(), not bracket indexing.
console.log('communities:', boundaries.community_count);
for (const [nodeId, communityId] of boundaries.cluster_assignments) {
  console.log(`${nodeId} -> ${communityId}`);
}
