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
| class_hierarchy | Code constructs that establish inheritance, interface implementation, or trait conformance relationships — e.g. classes with `extends`/`implements` clauses, Python classes with base classes, and Rust `impl Trait for Type` blocks. All declaration kinds are classified here regardless of whether they carry a heritage clause; heritage-aware narrowing is the embedder's responsibility. |
| data_access | Code constructs that perform I/O against data stores or external resources — e.g., database queries (`query`, `cursor.*`), HTTP fetches (`fetch`), file reads (`open`, `read`), and ORM method calls. All `call_expression` / `call` nodes are classified here; callee-name narrowing is the embedder's responsibility. |
| error_handling | Code constructs that propagate, transform, or handle error conditions — e.g., the `?` operator (`try_expression`) and `match` arms that dispatch on `Result` or `Option` variants. |
| logging | Code constructs that produce diagnostic or observability output — e.g., `console.*` calls, structured logger invocations (`logger.info`), `print` statements, and logging macros (`tracing::info!`, `log::debug!`). **Catalog-only in sdivi-rust v0** — see Embedder responsibilities. |
| resource_management | Code constructs that allocate, release, or manage system or heap resources — e.g., macro invocations such as `drop!`, `vec!`, or standard I/O macros. |
| state_management | Code constructs that capture, transform, or share mutable or shared state — e.g., closures that close over mutable bindings or shared references. |
| type_assertions | Code constructs that assert or coerce between types at compile or runtime — e.g., `as` casts (`as_expression`) and language-specific type-cast expressions. |

## Per-language node-kind mappings

Each cell lists the tree-sitter node-kind strings that map to that category in a given language. The current classification is language-unaware (the `language` parameter in `category_for_node_kind` is reserved for future per-language overrides). All supported languages share the same node-kind table.

### Rust

| Category | Node kinds | Structural constraint |
|---|---|---|
| async_patterns | `await_expression` | None |
| class_hierarchy | `impl_item` | All `impl` blocks, including inherent `impl Type {…}` (no trait) and trait conformance `impl Trait for Type {…}`. Inherent-only narrowing is the embedder's responsibility. |
| data_access | (none in v0) | — |
| error_handling | `try_expression`, `match_expression` | None (both `?` and `match` are counted; callers may apply finer-grained filters in their own extractors) |
| logging | (consumer extractor responsibility — `macro_invocation` overlaps with `resource_management` at the AST level) | — |
| resource_management | `macro_invocation` | None |
| state_management | `closure_expression` | None |
| type_assertions | `as_expression` | None |

### Python

| Category | Node kinds | Structural constraint |
|---|---|---|
| async_patterns | `await` | None |
| class_hierarchy | `class_definition` | All `class` definitions, including those with no base classes (which are effectively `class Foo(object)` and contribute low entropy). |
| data_access | `call` | None (all Python function calls; callee-name narrowing is the embedder's responsibility) |
| error_handling | `try_statement` | None |
| logging | (consumer extractor responsibility — `call` overlaps with `data_access` at the AST level) | — |
| resource_management | (none in v0) | — |
| state_management | `lambda` | None |
| type_assertions | (none in v0) | — |

### TypeScript / JavaScript

| Category | Node kinds | Structural constraint |
|---|---|---|
| async_patterns | `await_expression` | None |
| class_hierarchy | `class_declaration`, `abstract_class_declaration`, `interface_declaration` | Abstract classes and interfaces always count. Concrete classes count regardless of `extends` / `implements`; entropy survives the broader collection because heritage-free classes have similar structure and contribute low entropy. (JavaScript: only `class_declaration` is emitted; `interface_declaration` and `abstract_class_declaration` are TS-only AST shapes.) |
| data_access | `call_expression` | None (all call expressions; callee-name narrowing is the embedder's responsibility) |
| error_handling | `try_statement` | None |
| logging | (consumer extractor responsibility — `call_expression` overlaps with `data_access` at the AST level) | — |
| resource_management | (none in v0) | — |
| state_management | `arrow_function` | None |
| type_assertions | `type_cast_expression`, `as_expression` | None |

### Go / Java

These languages share the Rust classifier in v0 except for `data_access`, which maps `call_expression` across all languages. Language-specific refinements are deferred until concrete user feedback warrants them.

| Category | Node kinds | Structural constraint |
|---|---|---|
| class_hierarchy | Java: `class_declaration`, `interface_declaration`. Go: (none in v0 — Go has no class/interface AST shape; the duck-typed interface model does not surface as a `class_hierarchy` declaration. The category exists in the catalog so cross-language reporting is uniform, but it produces zero Go hits.) | Java: same broader-collection caveat as other languages — all declaration kinds are classified regardless of heritage. |
| data_access | `call_expression` | None (all call expressions; callee-name narrowing is the embedder's responsibility) |
| logging | (consumer extractor responsibility — `call_expression` overlaps with `data_access` at the AST level) | — |

> **Note on per-language node-kind tables:** The v0 tables above are written by hand.
> A future milestone could derive them from the tree-sitter query definitions to eliminate
> this doc/code drift surface. Until then, the [category contract test](../crates/sdivi-core/tests/category_contract.rs)
> is the authoritative drift detector for the category *set*; per-language node-kind
> accuracy relies on manual review.

## Callee-text classification (`classify_hint`)

As of M32, `sdivi_core::classify_hint(hint, language) -> Vec<String>` provides a
higher-precision classifier that inspects both the `node_kind` and `hint.text` (the
truncated source text of the node). Foreign extractors should prefer `classify_hint`
over hand-rolled callee filters — the regex tables below are part of the canonical
contract and are versioned with `snapshot_version "1.0"`.

The native `Pipeline::snapshot` continues to use `category_for_node_kind` in M32
(see M33 for the pipeline switchover). Snapshot output is bit-identical before and
after M32.

### `data_access::matches_callee(text, language)`

| Language | Pattern | Examples matched |
|---|---|---|
| TypeScript / JavaScript / Go | `^(fetch\|axios)\b` | `fetch("/api")`, `axios.get(url)` |
| TypeScript / JavaScript / Go | `\b(query\|read\|write\|get\|post\|put\|delete\|patch)\(` | `db.query(sql)`, `get(url)` |
| TypeScript / JavaScript / Go | `\b(db\|sql)\.` | `db.execute(sql)`, `sql.Open(dsn)` |
| TypeScript / JavaScript / Go | `\.(query\|read\|write\|fetch)\(` | `client.read(buf)`, `.fetch(url)` |
| Python | `^(open\(\|requests\.\|httpx\.\|cursor\.\|session\.\|conn\.)` | `open(path)`, `cursor.execute(q)` |
| Rust, Java | (none in v0) | — |

**Worked example (TypeScript):** `fetch("/api/users")` → `["data_access"]`

### `logging::matches_callee(text, language)`

| Language | Pattern | Examples matched |
|---|---|---|
| TypeScript / JavaScript | `^(console\|logger\|log)\.` | `console.log(x)`, `logger.info(x)` |
| Python | `^(logging\.\|print\b)` | `logging.info(x)`, `print(x)` |
| Go | `^fmt\.(Print\|Println\|Printf\|Errorf\|Fprint\|Sprint)` | `fmt.Println(x)`, `fmt.Printf(f, x)` |
| Rust | `^(tracing\|log)::\|^(println\|eprintln\|print\|eprint\|dbg)!` | `tracing::info!(x)`, `println!(x)` |
| Java | `^(System\.(out\|err)\.\|logger\.\|Log\.\|LOG\.)` | `System.out.println(x)`, `LOG.info(x)` |

**Worked example (Rust):** `tracing::info!("request")` → `["logging"]`

### `async_patterns::matches_callee(text, language)`

| Language | Pattern | Examples matched |
|---|---|---|
| TypeScript / JavaScript | `\.(then\|catch\|finally)\(` | `p.then(r)`, `fetch().catch(e => {})` |
| All others | (none) | — |

**Worked example (TypeScript):** `promise.then(resolve)` → `["async_patterns"]`

### `resource_management::excludes_callee(text, language)`

This function is **inverted**: returns `true` when a `macro_invocation` should fall
through to `logging` instead of staying in `resource_management`.

| Language | Pattern | Examples excluded |
|---|---|---|
| Rust | `^(tracing\|log)::\|^(println\|eprintln\|print\|eprint\|dbg)!` | `tracing::info!`, `println!` |
| All others | (none) | — |

**Worked example (Rust):** `vec![1, 2, 3]` macro invocation → `["resource_management"]`;
`tracing::info!("x")` macro invocation → `["logging"]`.

### Dispatch order in `classify_hint`

For `call_expression` / `call` node kinds:
1. `async_patterns` (highest priority) — `.then`, `.catch`, `.finally`
2. `logging` — `console.*`, `logger.*`, `tracing::*!`, etc.
3. `data_access` — `fetch`, `query`, `cursor.*`, etc.
4. `[]` empty — unrecognised callee, hint is dropped

For `macro_invocation`:
- Logging macros (Rust only) → `["logging"]`
- All others → `["resource_management"]`

All other node kinds fall through to `category_for_node_kind`.

### Regex change log

First defined in M32 for `snapshot_version "1.0"`. Changing or narrowing a regex
is a behavioural break requiring a `MIGRATION_NOTES.md` entry. Broadening (adding
new shapes) is additive.

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
5. **`data_access` covers all call nodes at the sdivi-rust layer.** The `data_access` category classifies every `call_expression` node (TypeScript, JavaScript, Go) and every `call` node (Python) — no callee-name filtering is applied at the sdivi-rust layer. Embedders that want callee-name precision (e.g. filtering to only `fetch`, `query`, or `cursor.*` calls) must filter `PatternInstanceInput` records themselves before calling `compute_pattern_metrics`.
6. **The `logging` category in `snapshot_version "1.0"` is catalog-only for `category_for_node_kind`.** `sdivi_patterns::queries::category_for_node_kind` never returns `Some("logging")`. **Updated in M32:** `classify_hint` *does* return `["logging"]` for matching callees — see the "Callee-text classification" section above. Embedders using `classify_hint` get canonical callee filtering for free. Embedders that do not use `classify_hint` MUST apply callee-text filtering on their side and pass `PatternInstanceInput { category: "logging", … }` directly into `compute_pattern_metrics`.
7. **The `class_hierarchy` category in `snapshot_version "1.0"` is wired natively but classified broadly** — every declaration of the listed node kinds is included regardless of heritage. Embedders that want heritage-only precision (e.g. only classes with an `extends` clause, only `impl Trait for …` blocks) should filter `PatternInstanceInput` on their side before passing to `compute_pattern_metrics`. Entropy-based divergence signals remain meaningful under the broader collection because hierarchy-free declarations contribute low structural variance — the signal is the variance introduced by hierarchical declarations, not the absolute count.

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
