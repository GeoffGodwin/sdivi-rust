'use strict';
// Validates that pkg-template/package.json is parseable JSON with the required
// conditional-exports map. This catches malformed templates before a publish.
// Run with: node tests/validate_pkg_template.cjs (no WASM build required).

const fs = require('fs');
const path = require('path');

let failed = false;

function fail(msg) {
    console.error('FAIL: ' + msg);
    failed = true;
}

function ok(msg) {
    console.log('OK: ' + msg);
}

// ── 1. pkg-template/package.json ──────────────────────────────────────────────

const templatePath = path.resolve(__dirname, '../pkg-template/package.json');
const rawTemplate = fs.readFileSync(templatePath, 'utf8');

let pkg;
try {
    pkg = JSON.parse(rawTemplate);
} catch (e) {
    fail('pkg-template/package.json is not valid JSON: ' + e.message);
    process.exit(1);
}
ok('pkg-template/package.json parses as valid JSON');

// exports field must exist and be an object
if (!pkg.exports || typeof pkg.exports !== 'object' || Array.isArray(pkg.exports)) {
    fail('"exports" field is missing or not an object');
} else {
    ok('"exports" field present and is an object');

    // Root "." entry must have both "import" and "require" keys
    const root = pkg.exports['.'];
    if (!root || typeof root !== 'object') {
        fail('exports["."] is missing or not an object');
    } else {
        if (typeof root['import'] !== 'string') {
            fail('exports["."]["import"] is missing or not a string');
        } else {
            ok('exports["."]["import"] = ' + root['import']);
        }
        if (typeof root['require'] !== 'string') {
            fail('exports["."]["require"] is missing or not a string');
        } else {
            ok('exports["."]["require"] = ' + root['require']);
        }
    }

    // "./node" subpath must exist
    if (!pkg.exports['./node'] || typeof pkg.exports['./node'] !== 'object') {
        fail('exports["./node"] subpath is missing or not an object');
    } else {
        ok('exports["./node"] subpath present');
    }

    // "./bundler" subpath must exist
    if (!pkg.exports['./bundler'] || typeof pkg.exports['./bundler'] !== 'object') {
        fail('exports["./bundler"] subpath is missing or not an object');
    } else {
        ok('exports["./bundler"] subpath present');
    }

    // "import" path must reference bundler target; "require" must reference node target
    const imp = (pkg.exports['.'] || {})['import'] || '';
    const req = (pkg.exports['.'] || {})['require'] || '';
    if (!imp.includes('bundler/')) {
        fail('exports["."]["import"] does not reference bundler/ target: ' + imp);
    } else {
        ok('exports["."]["import"] references bundler/ target');
    }
    if (!req.includes('node/')) {
        fail('exports["."]["require"] does not reference node/ target: ' + req);
    } else {
        ok('exports["."]["require"] references node/ target');
    }

    // The "node" environment condition must route Node consumers (both ESM
    // and CJS) to the nodejs wasm-pack target. Without this, Node ESM
    // consumers resolve via "import" to the bundler target, which can't be
    // loaded by raw Node — `import * as wasm from './foo.wasm'` fails with
    // ERR_UNKNOWN_FILE_EXTENSION since wasm-bindgen's bundler output relies
    // on bundler-specific module handling.
    const nodeCond = (pkg.exports['.'] || {})['node'];
    if (typeof nodeCond !== 'string') {
        fail('exports["."]["node"] is missing or not a string — Node ESM consumers will fail');
    } else if (!nodeCond.includes('node/')) {
        fail('exports["."]["node"] does not reference node/ target: ' + nodeCond);
    } else {
        ok('exports["."]["node"] = ' + nodeCond + ' (routes Node ESM+CJS to nodejs target)');
    }
}

// engines.node must be >=18
if (!pkg.engines || !pkg.engines.node) {
    fail('"engines.node" field is missing');
} else {
    ok('"engines.node" = ' + pkg.engines.node);
    // Must be parseable as a semver range anchored at >=18
    if (!pkg.engines.node.includes('18') && !pkg.engines.node.match(/>=\s*1[89]|>=\s*[2-9]/)) {
        fail('"engines.node" does not declare >=18 minimum: ' + pkg.engines.node);
    } else {
        ok('"engines.node" declares a minimum of >=18');
    }
}

// ── 2. tests/node_smoke/package.json ─────────────────────────────────────────

const smokePath = path.resolve(__dirname, 'node_smoke/package.json');
const rawSmoke = fs.readFileSync(smokePath, 'utf8');

let smoke;
try {
    smoke = JSON.parse(rawSmoke);
} catch (e) {
    fail('tests/node_smoke/package.json is not valid JSON: ' + e.message);
    process.exit(1);
}
ok('tests/node_smoke/package.json parses as valid JSON');

// Smoke test engines.node must also declare >=18
if (!smoke.engines || !smoke.engines.node) {
    fail('node_smoke/package.json missing "engines.node"');
} else {
    ok('node_smoke/package.json engines.node = ' + smoke.engines.node);
    if (!smoke.engines.node.includes('18') && !smoke.engines.node.match(/>=\s*1[89]|>=\s*[2-9]/)) {
        fail('node_smoke/package.json "engines.node" does not declare >=18 minimum');
    } else {
        ok('node_smoke/package.json "engines.node" consistent with >=18 minimum');
    }
}

// npm test script must not use stdin redirect for ESM (aligns with CI invocation)
if (smoke.scripts && smoke.scripts.test) {
    const testScript = smoke.scripts.test;
    if (testScript.includes('--input-type=module <')) {
        fail(
            'node_smoke npm test script uses stdin redirect for ESM (' +
            testScript +
            ') — CI uses `node index.mjs` directly; they should match'
        );
    } else {
        ok('node_smoke npm test script does not use stdin redirect: ' + testScript);
    }
}

// ── 3. Result ─────────────────────────────────────────────────────────────────

if (failed) {
    console.error('\nvalidate_pkg_template: FAILED');
    process.exit(1);
}
console.log('\nvalidate_pkg_template: all checks passed');
