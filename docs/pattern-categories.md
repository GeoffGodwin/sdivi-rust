# Pattern Category Contract

**Applies to:** `snapshot_version "1.0"`

## Versioning

Pattern categories are bound to `snapshot_version "1.0"`. The rules are:

- **Reserved forever once introduced.** A category name that appears in `list_categories()` cannot be removed within a snapshot version — only marked deprecated in its description. Embedders may have stored snapshots referencing it, and `compute_delta` must keep working.
- **Additive within a snapshot version.** New categories may be added; existing names and their meanings may not change.
- **Removed-category procedure.** If a category must truly be retired (not just deprecated), that requires bumping `snapshot_version` to a new value per Rule 16 of CLAUDE.md. Document the bump procedure in `MIGRATION_NOTES.md`.
- **Per-language node-kind tables are an implementation detail.** The `snapshot_version "1.0"` contract covers the *category set*. Node-kind strings can grow as new language adapters are added without a version bump.

The authoritative runtime source of truth is `sdivi_core::list_categories()`. The table below is generated from the same constant — the [category contract test](../crates/sdivi-core/tests/category_contract.rs) asserts they match.

## Canonical category list

| Category | Description |
|---|---|
| async_patterns | Code constructs that implement or leverage asynchronous execution — e.g., `.await` expressions on `Future` values and `async fn` definitions. |
| error_handling | Code constructs that propagate, transform, or handle error conditions — e.g., the `?` operator (`try_expression`) and `match` arms that dispatch on `Result` or `Option` variants. |
| resource_management | Code constructs that allocate, release, or manage system or heap resources — e.g., macro invocations such as `drop!`, `vec!`, or standard I/O macros. |
| state_management | Code constructs that capture, transform, or share mutable or shared state — e.g., closures that close over mutable bindings or shared references. |
| type_assertions | Code constructs that assert or coerce between types at compile or runtime — e.g., `as` casts (`as_expression`) and language-specific type-cast expressions. |

## Per-language node-kind mappings

Each cell lists the tree-sitter node-kind strings that map to that category in a given language. The current classification is language-unaware (the `language` parameter in `category_for_node_kind` is reserved for future per-language overrides). All supported languages share the same node-kind table.

### Rust

| Category | Node kinds | Structural constraint |
|---|---|---|
| async_patterns | `await_expression` | None |
| error_handling | `try_expression`, `match_expression` | None (both `?` and `match` are counted; callers may apply finer-grained filters in their own extractors) |
| resource_management | `macro_invocation` | None |
| state_management | `closure_expression` | None |
| type_assertions | `as_expression` | None |

### Python

| Category | Node kinds | Structural constraint |
|---|---|---|
| async_patterns | `await` | None |
| error_handling | `try_statement` | None |
| resource_management | (none in v0) | — |
| state_management | `lambda` | None |
| type_assertions | (none in v0) | — |

### TypeScript / JavaScript

| Category | Node kinds | Structural constraint |
|---|---|---|
| async_patterns | `await_expression` | None |
| error_handling | `try_statement` | None |
| resource_management | (none in v0) | — |
| state_management | `arrow_function` | None |
| type_assertions | `type_cast_expression`, `as_expression` | None |

### Go / Java

These languages share the Rust classifier in v0. Language-specific refinements are deferred until concrete user feedback warrants them.

> **Note on per-language node-kind tables:** The v0 tables above are written by hand.
> A future milestone could derive them from the tree-sitter query definitions to eliminate
> this doc/code drift surface. Until then, the [category contract test](../crates/sdivi-core/tests/category_contract.rs)
> is the authoritative drift detector for the category *set*; per-language node-kind
> accuracy relies on manual review.

## Normalization rules

Pattern fingerprints are computed by `sdivi_core::normalize_and_hash`. The algorithm is:

1. **Input**: a `node_kind: &str` and an ordered `children: &[NormalizeNode]` slice.
2. **Leaf node** (`children` is empty): `blake3::keyed_hash(FINGERPRINT_KEY, node_kind.as_bytes())` — byte-identical to `fingerprint_node_kind(node_kind)`.
3. **Internal node**: input bytes = `node_kind.as_bytes()` + `0x00` + for each child: `0x01` + 32 child-digest bytes.
4. **Key**: `FINGERPRINT_KEY` — a 32-byte constant defined in `sdivi_patterns::fingerprint::FINGERPRINT_KEY` and re-exported from `sdivi_core::FINGERPRINT_KEY`. The key is **fixed for all `snapshot_version "1.0"` output**. Changing the key invalidates all existing snapshot fingerprints.

### Embedder responsibilities

An embedder that supplies `PatternInstanceInput` values must:

1. Use category names **verbatim** as returned by `list_categories()`. The comparison in `compute_pattern_metrics` is case-sensitive.
2. Compute fingerprints via `normalize_and_hash(node_kind, children)` (Rust) or the WASM export `normalize_and_hash(nodeKind, children)`. Do not implement a custom fingerprint algorithm.
3. When calling `normalize_and_hash`, pass the tree-sitter `node_kind` string and, if available, the ordered child subtree. For v0 language adapters, children is always empty — leaf-level fingerprints only.
4. The fingerprint must be a 64-character lowercase hex string as returned by `normalize_and_hash`.

Cross-runtime determinism: the WASM `normalize_and_hash` produces **bit-identical** output to the native Rust pipeline for the same input. See `docs/determinism.md` for the full guarantee.

## Runtime discovery (recommended)

Call `list_categories()` at startup rather than hard-coding category names:

```rust
use sdivi_core::list_categories;

let catalog = list_categories();
for cat in &catalog.categories {
    println!("{}: {}", cat.name, cat.description);
}
```

From WASM / TypeScript:

```ts
import init, { list_categories } from '@geoffgodwin/sdivi-wasm';

await init();
const catalog = list_categories();
console.log(catalog.schema_version); // "1.0"
for (const cat of catalog.categories) {
    console.log(cat.name, '-', cat.description);
}
```
