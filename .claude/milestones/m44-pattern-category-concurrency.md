
#### Milestone 44: Pattern Category — `concurrency`

<!-- milestone-meta
id: "44"
status: "planned"
-->

**Scope:** Add the `concurrency` category for concurrent-execution primitives that
are distinct from `async_patterns` (await/Promise). Go: `go_statement`
(goroutines), `select_statement` (channel multiplexing). These are **already
collected** by the Go adapter (`crates/sdivi-lang-go/src/extract.rs`) and currently
dropped — a free win. Optional callee-text extensions: `tokio::spawn` /
`thread::spawn` (Rust), `Promise.all`/`allSettled`/`race` (TS/JS),
`asyncio.gather` (Python). Node-kind path needs no parsing change; callee
extensions depend on M34.

**Why this milestone exists:** Goroutines and channels are *the* idiomatic Go
concurrency constructs and are presently parsed and thrown away. Their
density/shape is the central structural signal in Go services. Surfacing them
makes SDIVI meaningfully useful for Go, and keeps `concurrency` cleanly separate
from `async_patterns` (single-future await) and `state_management` (closures).

**Deliverables:**
- Create `crates/sdivi-patterns/src/queries/concurrency.rs`:
  `NODE_KINDS: &[&str] = &["go_statement", "select_statement"]`, plus `matches_callee`
  for the TS/JS `Promise.*` and Python `asyncio.*` forms (finalized below).
- Register in `ALL_CATEGORIES`, `category_for_node_kind`, the M34 `CALL_DISPATCH`
  registry at **slot P11 (lowest)**, and `CATALOG_ENTRIES`.
- Update `docs/pattern-categories.md` (Go table + callee table).

**Detection (finalized):**
| Language | Node kinds / Pattern | Examples matched |
|---|---|---|
| Go | node-kind `go_statement`, `select_statement` (core, no parsing change) | `go worker(ch)`, `select { case ... }` |
| TS / JS | callee `^Promise\.(all\|allSettled\|race\|any)\(` (M34 slot **P11**, lowest) | `Promise.all([...])` |
| Python | callee `^asyncio\.(gather\|create_task\|wait\|as_completed\|run)\(` | `asyncio.gather(*t)` |

**Rust deferred:** `tokio::spawn`/`thread::spawn` are `call_expression` nodes, but
the Rust adapter's `PATTERN_KINDS` does not collect `call_expression` (only macros,
match, try, await, closure, impl). Detecting them needs a Rust-adapter change —
out of scope here; recorded as a Seed. Do not add the Rust regex without first
adding `call_expression` to the Rust adapter (a larger change with its own
migration impact).

**Migration Impact:** Additive; `list_categories()` +1. Go repos gain a non-zero
bucket (previously-dropped hints). If the TS/JS `Promise.all` extension is included,
those calls move from `[]` (unrecognised) to `concurrency` — no existing category
loses them. `snapshot_version` stays `"1.0"`. `MIGRATION_NOTES.md` entry.

**Files to create or modify:**
- **Create:** `crates/sdivi-patterns/src/queries/concurrency.rs`.
- **Modify:** `crates/sdivi-patterns/src/queries/mod.rs`,
  `crates/sdivi-core/src/categories.rs`.
- **Modify:** `docs/pattern-categories.md`, `MIGRATION_NOTES.md`, `CHANGELOG.md`.
- **Verify only:** Go adapter already collects `go_statement`/`select_statement`.

**Acceptance criteria:**
- `category_for_node_kind("go_statement", "go") == Some("concurrency")`.
- A Go fixture with goroutines + a `select` yields `concurrency` instances.
- `category_contract.rs`, WASM count test, clippy/fmt/doc gates green.

**Tests:**
- Unit: `go_statement`/`select_statement` classification; callee extensions if included.
- Integration: Go fixture count.

**Watch For:**
- **Boundary with `async_patterns`.** Keep `await_expression`/Promise-chains in
  `async_patterns`; goroutines/channels/`Promise.all` in `concurrency`. Document the
  split so the two buckets stay conceptually clean.
- **`defer_statement` is NOT concurrency** — it is resource/cleanup; it belongs to
  M45.1 (`resource_management`). Do not absorb it here.
- **Scope the optional callee extensions deliberately.** Shipping Go-only first
  (pure node-kind, zero risk) and adding spawn/gather later is a valid split if the
  regex set needs more bake time.

**Seeds Forward:**
- Channel send/receive operators (`ch <- x`, `<-ch`) and `sync.WaitGroup`/`Mutex`
  usage are deeper Go concurrency signals requiring operator/receiver extraction.
  Defer.
