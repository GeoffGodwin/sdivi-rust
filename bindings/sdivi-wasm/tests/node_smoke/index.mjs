// ESM smoke test — exercises the `import` conditional export path.
// In CI this file is run with: node index.mjs
// The package.json `exports["."]["import"]` resolves to the bundler wasm-pack target.
// On Node 18+ the bundler target works via dynamic import() of the .wasm file.

import init, { list_categories } from '@geoffgodwin/sdivi-wasm';

// The bundler target requires an explicit init() call to load the .wasm.
await init();

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
