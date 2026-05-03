'use strict';
// CommonJS smoke test — exercises the `require` conditional export path.
// In CI this file is run with: node index.cjs
// The package.json `exports["."]["require"]` resolves to the nodejs wasm-pack target,
// which uses require('fs') to load the .wasm synchronously — no WebAssembly shim needed.

const pkg = require('@geoffgodwin/sdivi-wasm');

// The nodejs wasm-pack target auto-initialises synchronously; no init() call needed.
const catalog = pkg.list_categories();

if (!catalog || !Array.isArray(catalog.categories)) {
    console.error('FAIL [CJS]: list_categories() did not return a catalog with categories array');
    process.exit(1);
}
if (catalog.schema_version !== '1.0') {
    console.error('FAIL [CJS]: expected schema_version "1.0", got: ' + catalog.schema_version);
    process.exit(1);
}
if (catalog.categories.length === 0) {
    console.error('FAIL [CJS]: list_categories() returned empty categories array');
    process.exit(1);
}

// Stash for cross-test comparison (written to stdout so CI can compare with ESM output).
console.log('CJS_CATEGORIES:' + JSON.stringify(catalog.categories.map(c => c.name).sort()));
console.log('OK [CJS]: list_categories() returned ' + catalog.categories.length + ' categories');
