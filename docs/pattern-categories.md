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
| async_patterns | Code constructs that implement or leverage asynchronous execution — e.g., `.await` expressions on `Future` values and `async fn` definitions. In TypeScript/JavaScript, Promise-chain calls (`.then()`, `.catch()`, `.finally()`) are also classified here via callee-text inspection. |
| class_hierarchy | Code constructs that establish inheritance, interface implementation, or trait conformance relationships — e.g. classes with `extends`/`implements` clauses, Python classes with base classes, and Rust `impl Trait for Type` blocks. All declaration kinds are classified here regardless of whether they carry a heritage clause; heritage-aware narrowing is the embedder's responsibility. |
| data_access | Code constructs that perform I/O against data stores or external resources — e.g., database queries (`db.query`, `cursor.*`), HTTP fetches (`fetch`, `axios`), file reads (`open`, `requests.*`). As of M33, only `call_expression`/`call` nodes whose callee text matches the per-language data-access regex are classified here; unrecognised callees are dropped. |
| decorators | Decorator usage across languages. TypeScript/JavaScript: any `decorator` node (`@Injectable()`, `@Component({...})`, `@Entity()`, `@Get('/')`, `@IsString()`, etc.) — one instance per decorator line. Python: any `decorated_definition` wrapper node (`@dataclass`, `@property`, `@app.get(...)`, `@pytest.fixture`, `@cached_property`, etc.) — one instance per decorated function or class (wrapper-granularity). Added M36.1 (TS/JS); M36.2 (Python). |
| error_handling | Code constructs that propagate, transform, or handle error conditions — e.g., the `?` operator (`try_expression`) and `match` arms that dispatch on `Result` or `Option` variants. |
| framework_hooks | Component-composition hook calls in React, Preact, Vue (composables), and Svelte-style runtimes — any `call_expression` callee matching `^use[A-Z]` in TypeScript or JavaScript. Covers built-in hooks (`useState`, `useEffect`, `useMemo`, `useCallback`, `useRef`, `useContext`, `useReducer`, `useLayoutEffect`) and the full custom-hook ecosystem. Added M35. |
| logging | Code constructs that produce diagnostic or observability output — e.g., `console.*` calls, structured logger invocations (`logger.info`), `print` statements, and logging macros (`tracing::info!`, `log::debug!`). **Natively classified since M33** via callee-text inspection in `classify_hint` — see Callee-text classification section. |
| resource_management | Code constructs that allocate, release, or manage system or heap resources — e.g., Rust macro invocations such as `drop!`, `vec!`, `assert!`. As of M33, Rust logging macros (`tracing::*!`, `log::*!`, `println!`/`eprintln!`/`print!`/`eprint!`/`dbg!`) are excluded and classified as `logging` instead. |
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
| logging | `macro_invocation` where callee matches `^(tracing\|log)::\|^(println\|eprintln\|print\|eprint\|dbg)!` | Natively classified since M33 via `classify_hint`. Examples: `tracing::info!(x)`, `log::debug!(x)`, `println!(x)`. |
| resource_management | `macro_invocation` where callee does NOT match the logging regex above | Logging macros are excluded and routed to `logging` instead. Remaining: `vec!`, `assert!`, `drop!`, `format!`, etc. |
| state_management | `closure_expression` | None |
| type_assertions | `as_expression` | None |

### Python

| Category | Node kinds | Structural constraint |
|---|---|---|
| async_patterns | `await` | None |
| class_hierarchy | `class_definition` | All `class` definitions, including those with no base classes (which are effectively `class Foo(object)` and contribute low entropy). |
| data_access | `call` where callee matches `^(open\(\|requests\.\|httpx\.\|cursor\.\|session\.\|conn\.)` | Natively filtered since M33. Examples: `open(path)`, `cursor.execute(q)`, `requests.get(url)`. Unrecognised calls (e.g. `len(x)`) are dropped. |
| decorators | `decorated_definition` | One instance per decorated function or class definition — wrapper-granularity. Three stacked `@`-lines on one function = one instance. Contrast with TypeScript/JavaScript, which counts one instance per `decorator` line. Added M36.2. |
| error_handling | `try_statement` | None |
| logging | `call` where callee matches `^(logging\.\|print\b)` | Natively classified since M33. Examples: `logging.info(x)`, `print(x)`. |
| resource_management | (none in v0) | — |
| state_management | `lambda` | None |
| type_assertions | (none in v0) | — |

### TypeScript / JavaScript

| Category | Node kinds | Structural constraint |
|---|---|---|
| async_patterns | `await_expression`; `call_expression` where callee matches `\.(then\|catch\|finally)\(` | `await_expression` via node-kind; Promise chains via callee-text. Examples: `future.await`, `p.then(r)`, `fetch().catch(e => {})`. |
| class_hierarchy | `class_declaration`, `abstract_class_declaration`, `interface_declaration` | Abstract classes and interfaces always count. Concrete classes count regardless of `extends` / `implements`; entropy survives the broader collection because heritage-free classes have similar structure and contribute low entropy. (JavaScript: only `class_declaration` is emitted; `interface_declaration` and `abstract_class_declaration` are TS-only AST shapes.) |
| data_access | `call_expression` where callee matches `^(fetch\|axios)\b\|\b(query\|read\|write\|get\|post\|put\|delete\|patch)\(\|\b(db\|sql)\.\|\.(query\|read\|write\|fetch)\(` | Natively filtered since M33. Examples: `fetch(url)`, `db.query(sql)`. Unrecognised calls (e.g. `Math.max(a, b)`) are dropped. |
| decorators | `decorator` | Natively classified since M36.1. Examples: `@Injectable()`, `@Component({...})`, `@Entity()`, `@Get('/')`, `@IsString()`. Node-kind only — all decorators count. |
| error_handling | `try_statement` | None |
| framework_hooks | `call_expression` where callee matches `^use[A-Z]` | Natively classified since M35. Examples: `useState(0)`, `useEffect(fn, [])`, `useAuth()`. Second character must be uppercase — `user()` does not match. |
| logging | `call_expression` where callee matches `^(console\|logger\|log)\.` | Natively classified since M33. Examples: `console.log(x)`, `logger.info(x)`. |
| resource_management | (none in v0) | — |
| state_management | `arrow_function` | None |
| type_assertions | `type_cast_expression`, `as_expression` | None |

### Go / Java

These languages share the common callee-text filter via `classify_hint`. Go and Java `call_expression` nodes are filtered the same way as TypeScript/JavaScript for `data_access` (shared regex table); logging uses per-language regex tables.

| Category | Node kinds | Structural constraint |
|---|---|---|
| class_hierarchy | Java: `class_declaration`, `interface_declaration`. Go: (none in v0 — Go has no class/interface AST shape; the duck-typed interface model does not surface as a `class_hierarchy` declaration. The category exists in the catalog so cross-language reporting is uniform, but it produces zero Go hits.) | Java: same broader-collection caveat as other languages — all declaration kinds are classified regardless of heritage. |
| data_access | `call_expression` where callee matches the shared TS/JS/Go regex (`^(fetch\|axios)\b\|\b(db\|sql)\.` etc.) | Natively filtered since M33. Examples: `db.query(sql)`, `sql.Open(dsn)`. Java `call_expression` returns `false` in v0 — data-access detection is library-shaped and deferred. |
| logging | Go: `call_expression` where callee matches `^fmt\.(Print\|Println\|Printf\|Errorf\|Fprint\|Sprint)`. Java: `call_expression` where callee matches `^(System\.(out\|err)\.\|logger\.\|Log\.\|LOG\.)` | Natively classified since M33. Go examples: `fmt.Println(x)`, `fmt.Printf(f, x)`. Java examples: `System.out.println(x)`, `LOG.info(x)`. |

> **Note on per-language node-kind tables:** The v0 tables above are written by hand.
> A future milestone could derive them from the tree-sitter query definitions to eliminate
> this doc/code drift surface. Until then, the [category contract test](../crates/sdivi-core/tests/category_contract.rs)
> is the authoritative drift detector for the category *set*; per-language node-kind
> accuracy relies on manual review.

## Callee-text classification (`classify_hint`)

`sdivi_core::classify_hint(hint, language) -> Vec<String>` provides a
higher-precision classifier that inspects both the `node_kind` and `hint.text` (the
truncated source text of the node). Foreign extractors should prefer `classify_hint`
over hand-rolled callee filters — the regex tables below are part of the canonical
contract and are versioned with `snapshot_version "1.0"`.

**As of M33, the regex tables are load-bearing for native pipeline output, not just
embedder convenience.** `Pipeline::snapshot` now calls `classify_hint` instead of
`category_for_node_kind`. Per-category instance counts shift on upgrade — see
`MIGRATION_NOTES.md` for the M33 migration story and a worked example.

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

### `framework_hooks::matches_callee(text, language)`

| Language | Pattern | Examples matched |
|---|---|---|
| TypeScript / JavaScript | `^use[A-Z]` | `useState(0)`, `useEffect(fn, [])`, `useAuth()`, `useStore()` |
| All others | (none) | — |

**Worked example (TypeScript):** `useState(0)` → `["framework_hooks"]`

**Note:** The second character must be uppercase — `user()`, `useful()` do not match. The anchor `^` prevents mid-identifier matches.

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

`classify_hint`'s `call_expression`/`call` arm iterates the `CALL_DISPATCH` registry
(`crates/sdivi-patterns/src/queries/mod.rs`). **First match wins.** The order below
is the contract — future milestones insert at their named slot, never append.

#### Canonical precedence table

| Slot | Category | Active | Representative regex / pattern |
|---|---|---|---|
| P1 | `async_patterns` | M34 | `\.(then\|catch\|finally)\(` |
| P2 | `testing` | M42 | `^(describe\|it\|test\|expect)\(`, `^jest\.` |
| P3 | `serialization` | M43 | `^JSON\.(parse\|stringify)\(`, `^json\.(Marshal\|Unmarshal)\(` |
| P4 | `schema_validation` | M38 | `^(z\|yup\|v\|s)\.\w`, `\.safeParse\(`, `\bBaseModel\b` |
| P5 | `state_store` | M39 | redux/zustand/jotai factories; `^use(Selector\|Dispatch\|Store)\b` |
| P6 | `framework_hooks` | M35 | `^use[A-Z]` |
| P7 | `http_routing` | M41 | `^(app\|router\|fastify\|server\|srv)\.(get\|post\|…)\(` |
| P8 | `logging` | M34 | `^(console\|logger\|log)\.`, `^fmt\.Print`, `^(tracing\|log)::` |
| P9 | `data_access` | M34 | `^(fetch\|axios)\b`, `\b(db\|sql)\.`, `cursor\.`, `requests\.` |
| P10 | `collection_pipelines` | M40 | `\.(map\|filter\|reduce\|flatMap\|forEach)\(` |
| P11 | `concurrency` | M44 | `^Promise\.(all\|allSettled\|race\|any)\(`, `^asyncio\.gather\(` |

P1, P6, P8, and P9 are active at M36.2. The `decorators` category is node-kind-only and does
not appear in `CALL_DISPATCH` — it is classified via `category_for_node_kind` in the
`other =>` arm of `classify_hint`. All other slots are reserved placeholders.

#### KNOWN_OVERLAPS policy

When a callee string legitimately matches two categories' regexes, the first-match
winner is correct by construction. The overlap must be documented in the
`KNOWN_OVERLAPS` table in `crates/sdivi-patterns/tests/dispatch_disjointness.rs`.

Documented overlaps at M35 (P1/P6/P8/P9 active):

| Callee | Language | Winner | Loser | Rationale |
|---|---|---|---|---|
| `fetch(url).catch(err => {})` | javascript | `async_patterns` | `data_access` | Chained-fetch outer node matches both `\.(catch)\(` (P1) and `^fetch\b` (P9); P1 wins by precedence |

Future overlaps introduced by M39–M44:
- `useSelector` / `useDispatch` / `useStore` → **state_store** (P5) beats `framework_hooks` (P6). More specific wins. Until M39 lands, these resolve to `framework_hooks` — acceptable.
- `app.get` / `router.post` → **http_routing** (P7) beats `data_access` (P9). Client fetches (`axios.get`) stay in `data_access` — the http_routing receiver allowlist excludes them.
- `Promise.all([]).then(cb)` outer node → **async_patterns** (P1) wins; bare inner `Promise.all(…)` resolves to `concurrency` (P11).

#### `macro_invocation` arm

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
5. **`data_access` is callee-filtered since M33.** Only `call_expression`/`call` nodes whose callee text matches the per-language data-access regex are classified here. Embedders that supply `PatternInstanceInput { category: "data_access", … }` directly continue to work — their instances merge with natively classified ones. Embedders that want custom callee filters should apply them before calling `compute_pattern_metrics`.
6. **As of M33, the `logging` category is natively classified by the pipeline via `classify_hint`.** `sdivi_patterns::queries::category_for_node_kind` still never returns `Some("logging")` — that sentinel is unchanged — but the native pipeline now calls `classify_hint`, which routes matching callees to `logging`. Embedders that pass `PatternInstanceInput { category: "logging" }` directly will continue to round-trip — their instances merge with the natively-classified ones in `compute_pattern_metrics` output. Embedders that previously hand-rolled their own logging filter should consider switching to `classify_hint` (M32) to stay aligned with the canonical regex set.
7. **As of M35, the `framework_hooks` category is natively classified for TypeScript and JavaScript** via `classify_hint` callee-text inspection (`^use[A-Z]` regex). Hook calls that were previously unrecognised (dropped to `[]`) are now counted in the `framework_hooks` bucket. On the first post-M35 snapshot of a TS/JS repo, `framework_hooks` transitions from zero to non-zero. This is a count-introduction event; see `MIGRATION_NOTES.md` for details.
8. **As of M36.1, the `decorators` category is natively classified for TypeScript and JavaScript** via the `decorator` tree-sitter node kind. `@Injectable()`, `@Component({...})`, `@Entity()`, `@Get('/')`, and any other decorator node are counted. On the first post-M36.1 snapshot of a TS/JS repo with decorators, the `decorators` bucket transitions from zero to non-zero — a count-introduction event; see `MIGRATION_NOTES.md`. **As of M36.2, Python is also supported** via the `decorated_definition` node kind (`@dataclass`, `@property`, `@app.get(...)`, `@pytest.fixture`, etc.). Count semantics differ: Python counts one instance per decorated definition (wrapper-granularity); TypeScript/JavaScript count one per decorator line; see `MIGRATION_NOTES.md`.
9. **The `class_hierarchy` category in `snapshot_version "1.0"` is wired natively but classified broadly** — every declaration of the listed node kinds is included regardless of heritage. Embedders that want heritage-only precision (e.g. only classes with an `extends` clause, only `impl Trait for …` blocks) should filter `PatternInstanceInput` on their side before passing to `compute_pattern_metrics`. Entropy-based divergence signals remain meaningful under the broader collection because hierarchy-free declarations contribute low structural variance — the signal is the variance introduced by hierarchical declarations, not the absolute count.

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
