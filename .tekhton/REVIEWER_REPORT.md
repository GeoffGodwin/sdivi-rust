# Reviewer Report — M25 Adapter Module-Specifier Extraction

## Verdict
APPROVED_WITH_NOTES

## Complex Blockers (senior coder)
- None

## Simple Blockers (jr coder)
- None

## Non-Blocking Notes
- `crates/sdivi-lang-javascript/src/extract.rs:31-32` — doc comment claims `import("./utils") → ["./utils"]` for dynamic imports, but the corresponding test at lines 72-76 only asserts no-panic and explicitly disclaims the extraction ("specifier extracted when grammar supports it"). The promise in the docstring is not verified; either test the claim or soften the doc to "best-effort, grammar-version-dependent."
- `crates/sdivi-lang-java/src/extract.rs:79-80` — wildcard detection via text search (`contains(".*")`) rather than AST-node kind is acknowledged in the comment (grammar variation across versions), but the choice means a comment like `/* .* pattern */` embedded inside an unusual import declaration could produce a false wildcard. Low probability in practice; acceptable for MVP with the existing comment.
- `Docs Updated` in CODER_SUMMARY says "None — no public-surface changes." This is correct: `FeatureRecord` lives in `sdivi-parsing`, not `sdivi-core`, and is not in the documented public-API surface. The CHANGELOG and MIGRATION_NOTES entries both exist and are well-written.

## Coverage Gaps
- `crates/sdivi-lang-javascript/tests/extract_behavior.rs:72-76` — the `dynamic_import_string_literal_yields_specifier` test asserts only no-panic, not the extracted value. A grammar-aware assertion (or an explicit `assert!(record.imports.is_empty() || record.imports == &["./chunk.js"])`) would make the test's intent explicit rather than silent.
- No integration-test exercise of Python relative-import edge resolution (`from . import sibling`). `simple-python/` fixture uses only bare names and external stdlib; the end-to-end relative-dot case is covered only by the unit test of `relative_import_specifier` but not by the pinned-edge-count regression sentinel.
- The LOW-severity security finding from the security agent (`bindings/sdivi-wasm/src/weight_keys.rs:25-34` — missing `is_infinite()` guard on edge weights) is out of M25 scope but should be tracked for the next WASM-touching milestone. The fix is a one-liner with a matching test.

## Drift Observations
- `truncate_to_256_bytes` is defined identically (`pub(crate)`) in all five adapter `extract.rs` files (Python, TypeScript, JavaScript, Go, Java) and has inline unit tests only in the Python file. Factoring it into `sdivi-parsing` as a shared utility would eliminate the duplication; lower priority than M26 work but worth a cleanup ticket.
- `string_content` (unquote a `string` AST node) is byte-for-byte identical in `crates/sdivi-lang-typescript/src/extract.rs:74-100` and `crates/sdivi-lang-javascript/src/extract.rs:98-123`. Same consolidation opportunity as above.
- `crates/sdivi-parsing/tests/import_extraction.rs` is a cross-crate integration test (uses sdivi-graph + six language adapters) placed inside `sdivi-parsing/tests/`. CLAUDE.md calls for workspace-level cross-crate tests to live under the top-level `tests/` directory. Pragmatically fine here (`CARGO_MANIFEST_DIR` path resolution is the reason), but the pattern drifts from the stated convention.
