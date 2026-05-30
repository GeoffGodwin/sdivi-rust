
#### Milestone 37: Pattern Category — `null_safety`

<!-- milestone-meta
id: "37"
status: "planned"
-->

**Scope:** Add the `null_safety` pattern category for TypeScript / JavaScript —
optional chaining (`a?.b`, `a?.()`), non-null assertions (`a!`), and (deferred)
nullish coalescing (`a ?? b`). Optional chaining and non-null assertions are
distinct node kinds (`optional_chain`, `non_null_expression`); nullish coalescing
is an operator on `binary_expression` and is **deferred** (see Watch For).
**Requires a parsing-layer change:** neither kind is currently in the TS/JS
`PATTERN_KINDS`. Independent of the M34 call-dispatch framework (node-kind, not
callee-text).

**Why this milestone exists:** Null/undefined handling is one of the most visible
convention axes in TS — consistent `?.`/`??` adoption vs scattered `&&` guards and
`!` escape hatches. The mix and density of these operators is a clean,
judgment-free drift signal that is genuinely TS-defining, matching the user's
TS/JS-first priority.

**Deliverables:**
- Add `"optional_chain"` and `"non_null_expression"` to `PATTERN_KINDS` in
  `crates/sdivi-lang-typescript/src/extract.rs`; add `"optional_chain"` to
  `crates/sdivi-lang-javascript/src/extract.rs` (`non_null_expression` is TS-only).
- Create `crates/sdivi-patterns/src/queries/null_safety.rs`:
  `NODE_KINDS: &[&str] = &["optional_chain", "non_null_expression"]`, node-kind only.
- Register in `ALL_CATEGORIES`, `category_for_node_kind`, `CATALOG_ENTRIES`.
- Update `docs/pattern-categories.md` TS/JS table.

**Detection:**
| Language | Node kinds | Examples |
|---|---|---|
| TS | `optional_chain`, `non_null_expression` | `user?.name`, `arr?.[0]`, `fn?.()`, `el!` |
| JS | `optional_chain` | `user?.name`, `arr?.[0]`, `fn?.()` |

**Migration Impact:** Additive; `list_categories()` 10 → 11. TS/JS repos gain a
non-zero bucket; parsing stage emits more hints per file. No existing category
loses instances. `snapshot_version` stays `"1.0"`. `MIGRATION_NOTES.md` entry.

**Files to create or modify:**
- **Modify:** `crates/sdivi-lang-typescript/src/extract.rs`,
  `crates/sdivi-lang-javascript/src/extract.rs` — `PATTERN_KINDS`.
- **Create:** `crates/sdivi-patterns/src/queries/null_safety.rs`.
- **Modify:** `crates/sdivi-patterns/src/queries/mod.rs`,
  `crates/sdivi-core/src/categories.rs`.
- **Modify:** `docs/pattern-categories.md`, `MIGRATION_NOTES.md`, `CHANGELOG.md`.

**Acceptance criteria:**
- A TS fixture with `a?.b` and `c!` yields two `null_safety` instances.
- `category_for_node_kind("optional_chain", "typescript") == Some("null_safety")`.
- `category_contract.rs`, WASM count test, clippy/fmt/doc gates green.

**Tests:**
- Unit: both node kinds classify; non-match for python/go/rust/java.
- Integration: TS fixture mixing `?.`, `?.()`, `?.[i]`, `!` counts correctly.

**Watch For:**
- **Confirm node-kind strings against the pinned grammar.** tree-sitter-typescript
  represents optional chaining as `optional_chain` and non-null as
  `non_null_expression` — verify before wiring; grammar versions have moved these.
- **Nullish coalescing (`??`) is deferred.** It is a `binary_expression` with a
  `??` operator child, not a dedicated node kind. Detecting it cleanly needs
  operator-field inspection (the adapter would have to read the `operator` child
  and emit a synthetic hint), which is out of the v0 node-kind model. Scope it out;
  record as a Seed. Do **not** add `binary_expression` to `PATTERN_KINDS` — it is
  far too broad and would flood the catalog.
- **`optional_chain` granularity.** A long chain `a?.b?.c` may emit nested
  `optional_chain` nodes — decide and document whether that counts as 1 or N. Prefer
  counting each `optional_chain` node as-emitted for determinism; document it.

**Seeds Forward:**
- `??` support via an adapter-side operator-aware hint (synthetic `nullish_coalescing`
  node kind) when operator-level extraction is added — also unlocks logical-assignment
  operators (`??=`, `||=`, `&&=`). Defer until operator extraction exists.
