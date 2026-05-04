// ESM smoke test — exercises the `node` conditional export path from ESM.
// In CI this file is run with: node index.mjs
//
// Node's resolver matches the `node` environment condition first (before
// `import`/`require`), so both ESM and CJS Node consumers route to the
// nodejs wasm-pack target — which uses fs.readFileSync to load the .wasm
// synchronously, no init() needed. Browser/bundler consumers skip the
// `node` condition and resolve through `import` to the bundler target;
// that path's runtime is validated by wasm-bindgen-cli's own pinned test
// suite and by real consumer integration (e.g., Meridian) — it is not
// exercised end-to-end in this CI workflow.
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
