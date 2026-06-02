// ESM smoke test — exercises the `node` conditional export path from ESM.
// In CI this file is run with: node index.mjs
//
// Node's resolver matches the `node` environment condition first (before
// `import`/`require`), so both ESM and CJS Node consumers route to the
// nodejs wasm-pack target — which uses fs.readFileSync to load the .wasm
// synchronously, no init() needed. Browser/bundler consumers skip the
// `node` condition and resolve through `import` to the bundler target.
//
// Bundler *type* contract (both targets): validated by M47's tsc --noEmit
// guard over examples/binding_node.ts and examples/binding_bundler.ts against
// the freshly generated pkg/*.d.ts, plus a self-verifying negative fixture
// (tests/typecheck/negative.ts). Bundler *runtime* path (instantiation via
// wasm-pack bundler target): validated by wasm-bindgen-cli's own pinned test
// suite and by real consumer integration — a runtime bundler e2e is
// intentionally deferred (see M47 Non-Goals and Seeds Forward).
//
// The nodejs target is CJS, so we default-import then destructure.
// Node's cjs-module-lexer would also let us use named imports, but
// default-then-destructure is the most portable shape across Node versions.

import sdivi from '@geoffgodwin/sdivi-wasm';

const { list_categories } = sdivi;

const catalog = list_categories();

if (!catalog || !Array.isArray(catalog.categories)) {
    console.error('FAIL [ESM]: list_categories() did not return a catalog with categories array');
    process.exit(1);
}
if (catalog.schema_version !== '1.0') {
    console.error('FAIL [ESM]: expected schema_version "1.0", got: ' + catalog.schema_version);
    process.exit(1);
}
if (catalog.categories.length === 0) {
    console.error('FAIL [ESM]: list_categories() returned empty categories array');
    process.exit(1);
}

console.log('ESM_CATEGORIES:' + JSON.stringify(catalog.categories.map(c => c.name).sort()));
console.log('OK [ESM]: list_categories() returned ' + catalog.categories.length + ' categories');
