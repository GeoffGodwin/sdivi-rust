## Planned Tests
- [x] `bindings/sdivi-wasm/tests/test_check_docs.sh` — shell integration: doc-lint happy-path exits 0; forbidden-pattern detection exits 1 with FAIL message
- [x] `bindings/sdivi-wasm/tests/test_negative_integrity.sh` — structural: negative.ts has exactly 4 @ts-expect-error directives each immediately followed by code (no gap), and wasm.yml contains the required pinned steps
- [x] `bindings/sdivi-wasm/tests/typecheck/subpath_imports.ts` — TypeScript fixture importing via /bundler and /node subpaths to close coverage gap; update tsconfig include list

## Test Run Results
Passed: 46  Failed: 0

## Bugs Found
- BUG: [bindings/sdivi-wasm/tests/typecheck/negative.ts:70] Line contains `// @ts-expect-error directives above...` — TypeScript interprets any line beginning `// @ts-expect-error` as a suppression directive; since the immediately following `void _badEdgeWeights;` has no type error, this produces TS2578 "Unused '@ts-expect-error' directive" causing `tsc --noEmit` to exit non-zero. Verified: `npx tsc --noEmit -p bindings/sdivi-wasm/tests/typecheck/tsconfig.json` exits 2 before and after my changes.
- BUG: [bindings/sdivi-wasm/tests/typecheck/tsconfig.json] `"lib": ["ES2020"]` does not include `dom`; `console` global used throughout `examples/binding_node.ts` and `examples/binding_bundler.ts` is unavailable, producing TS2584 on every `console.log` call. The primary acceptance criterion ("tsc exits 0 with the corrected examples") fails due to this omission. Fix: add `"dom"` (or `"ES2020.Console"`) to the lib array, or replace `console.log` calls in the examples with alternative output.

## Files Modified
- [x] `bindings/sdivi-wasm/tests/test_check_docs.sh`
- [x] `bindings/sdivi-wasm/tests/test_negative_integrity.sh`
- [x] `bindings/sdivi-wasm/tests/typecheck/subpath_imports.ts`
- [x] `bindings/sdivi-wasm/tests/typecheck/tsconfig.json`
