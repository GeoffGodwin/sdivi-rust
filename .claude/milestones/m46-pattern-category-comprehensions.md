
#### Milestone 46: Pattern Category — `comprehensions`

<!-- milestone-meta
id: "46"
status: "planned"
-->

**Scope:** Add the `comprehensions` category for Python's comprehension family,
classifying node kinds the adapter **already collects and currently drops**:
`list_comprehension`, `set_comprehension`, `dictionary_comprehension`,
`generator_expression`. Pure node-kind classification — no parsing change, free win.
Python-only (no equivalent dedicated node kind in the other supported grammars).

**Why this milestone exists:** Comprehensions are a defining Python idiom and a real
style axis (comprehension vs explicit loop, nested vs flat, generator vs list). The
hints are parsed today and thrown away; routing them into a category is near-zero
cost and rounds out Python coverage alongside `decorators` (M36.2) and the M45
enrichments.

**Deliverables:**
- Create `crates/sdivi-patterns/src/queries/comprehensions.rs`:
  `NODE_KINDS: &[&str] = &["list_comprehension", "set_comprehension",
  "dictionary_comprehension", "generator_expression"]`, node-kind only.
- Register in `ALL_CATEGORIES`, `category_for_node_kind`, and `CATALOG_ENTRIES`.
- Update `docs/pattern-categories.md` Python table.

**Detection:**
| Language | Node kinds | Examples |
|---|---|---|
| Python | `list_comprehension`, `set_comprehension`, `dictionary_comprehension`, `generator_expression` | `[x for x in xs]`, `{k: v for ...}`, `(x for x in xs)` |

**Migration Impact:** Additive; `list_categories()` +1. Python repos gain a non-zero
bucket (previously-dropped hints); no existing category loses instances.
`snapshot_version` stays `"1.0"`. `MIGRATION_NOTES.md` entry.

**Files to create or modify:**
- **Create:** `crates/sdivi-patterns/src/queries/comprehensions.rs`.
- **Modify:** `crates/sdivi-patterns/src/queries/mod.rs`,
  `crates/sdivi-core/src/categories.rs`.
- **Modify:** `docs/pattern-categories.md`, `MIGRATION_NOTES.md`, `CHANGELOG.md`.
- **Verify only:** Python adapter already collects all four kinds.

**Acceptance criteria:**
- `category_for_node_kind("list_comprehension", "python") == Some("comprehensions")`.
- A Python fixture with list/dict/set/generator comprehensions yields four instances.
- `category_contract.rs`, WASM count test, clippy/fmt/doc gates green.

**Tests:**
- Unit: each of the four node kinds classifies; non-match for other languages.
- Integration: Python fixture count.

**Watch For:**
- **Confirm `generator_expression` node-kind string** in the pinned tree-sitter-python
  grammar — it is named `generator_expression`, not `*_comprehension`, but the
  adapter already lists it, so this is a verification, not a change.
- **Nested comprehensions** emit one node per comprehension; document the count rule.

**Seeds Forward:**
- TS/JS array-from-iterable idioms (`Array.from`, spread) are a loose analogue but
  shape-different; they belong to `collection_pipelines` (M40) reasoning, not here.
- Naming: `comprehensions` is Python-centric but reserved-forever — acceptable since
  the construct is genuinely Python-specific; cross-language consumers see an empty
  bucket for non-Python, consistent with how `class_hierarchy` reads empty for Go.
