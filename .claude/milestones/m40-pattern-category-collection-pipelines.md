
#### Milestone 40: Pattern Category — `collection_pipelines`

<!-- milestone-meta
id: "40"
status: "planned"
-->

**Scope:** Add the `collection_pipelines` category for functional collection
transforms: `.map`, `.filter`, `.reduce`, `.flatMap`, `.forEach`, `.find`,
`.some`, `.every`, `.flat`. Callee-text on `call_expression`; depends on M34. No
parsing-layer change. Primarily TS/JS; the same regex applies to any language whose
adapter collects `call_expression` (Go, Java) where these method names appear.

**Why this milestone exists:** Functional-vs-imperative iteration is one of the most
visible style axes in JS, and chain density/shape is a clean convention-drift
signal. High prevalence makes for a well-populated bucket.

**Deliverables:**
- Create `crates/sdivi-patterns/src/queries/collection_pipelines.rs` with the
  method-name regex.
- Register in `ALL_CATEGORIES`, the M34 `CALL_DISPATCH` registry at **slot P10
  (second-lowest, above only `concurrency`)** — it is broad, so every more-specific
  category resolves first. Add `CATALOG_ENTRIES`.
- Update `docs/pattern-categories.md`.

**Detection (finalized — method-name set is disjoint from the `data_access` and
`async_patterns` member-call regexes):**
| Language | Pattern | Examples matched |
|---|---|---|
| TS / JS | `\.(map\|filter\|reduce\|flatMap\|forEach\|find\|findIndex\|some\|every\|flat)\(` | `xs.map(f)`, `xs.filter(p).reduce(g, 0)` |

Disjointness check: `data_access` uses `\.(query\|read\|write\|fetch)\(` and
`async_patterns` uses `\.(then\|catch\|finally)\(` — no token here appears in either
set, so `collection_pipelines` is safely last among member-call categories.

**Migration Impact:** Additive; `list_categories()` +1. New non-zero bucket in
TS/JS (and possibly Go/Java) repos. **Potential overlap with `data_access`:** the
data-access regex includes `\b(read\|write\|fetch)\(` and `\.(query\|read\|write\|fetch)\(`
but **not** `map/filter/reduce` — disjoint. Verify in the M34 corpus.
`snapshot_version` stays `"1.0"`. `MIGRATION_NOTES.md` entry.

**Files to create or modify:**
- **Create:** `crates/sdivi-patterns/src/queries/collection_pipelines.rs`.
- **Modify:** `crates/sdivi-patterns/src/queries/mod.rs`,
  `crates/sdivi-core/src/categories.rs`.
- **Modify:** `docs/pattern-categories.md`, `MIGRATION_NOTES.md`, `CHANGELOG.md`.

**Acceptance criteria:**
- `xs.map(f)` → `["collection_pipelines"]`; `db.query(sql)` → `["data_access"]`.
- `category_contract.rs`, WASM count test, clippy/fmt/doc gates green.

**Tests:**
- Unit: each method-name positive; non-array `.map` (e.g. `new Map()`) tolerance note.
- Disjointness corpus extended; assert no overlap with `data_access`.

**Watch For:**
- **`.map` on `Map`/`Set`/RxJS observables.** Callee-text can't distinguish an
  array `.map` from `rxObservable.map` or a DOM `.forEach`. This is acceptable noise
  for an entropy measure — document it; do not attempt receiver-type analysis (that
  would require type info SDIVI deliberately does not compute).
- **Low precedence in the registry** so library-specific categories win first.

**Seeds Forward:**
- Pipe/compose utilities (`pipe(...)`, `compose(...)`, `flow(...)` from
  lodash/fp-ts/Ramda) are the same idiom family and could extend this regex later.
