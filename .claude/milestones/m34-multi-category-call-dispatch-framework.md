
#### Milestone 34: Multi-Category Call-Expression Dispatch Framework

<!-- milestone-meta
id: "34"
status: "planned"
-->

**Scope:** Refactor `classify_hint`'s `call_expression`/`call` arm from the current
hand-ordered `if`-chain (`async_patterns` > `logging` > `data_access`) into an
explicit, priority-ordered registry of callee-text categories, and add a
disjointness/precedence test harness. This is the enabling infrastructure for the
TS/JS pattern-category expansion (M35–M43): once many categories match
`call_expression`, the implicit ordering becomes fragile (`useSelector` is both a
hook and a store call; `axios.get` is both data-access and routing-shaped). A
single documented precedence list plus a test that asserts the per-language regex
tables either stay disjoint or have a documented winner is the only durable way to
keep `classify_hint` deterministic as categories grow.

**Why this milestone exists:** `classify_hint` (M32) was built for three
call-shaped categories. The expansion adds 6–7 more, all matching the same
`call_expression` node kind. Without a formal precedence model, every new category
risks silently stealing instances from an earlier one, and the count shifts become
impossible to reason about per-milestone. This milestone makes precedence a
first-class, tested contract before any category piles onto it. It introduces **no
new category** and **no output change** on its own — it is a pure refactor verified
by snapshot-identity.

**Deliverables:**
- Replace the inline `if`-chain in `crates/sdivi-patterns/src/queries/mod.rs`
  (`classify_hint`, `call_expression`/`call` arm) with iteration over an ordered
  `const CALL_DISPATCH: &[(&str, fn(&str, &str) -> bool)]` slice mapping
  category name → its `matches_callee` fn. First match wins; order = precedence.
- Document the precedence order and its rationale inline and in
  `docs/pattern-categories.md` ("Dispatch order in `classify_hint`" section).
- Add `crates/sdivi-patterns/tests/dispatch_disjointness.rs`: for a curated corpus
  of callee strings per language, assert each resolves to exactly the expected
  category, and assert that any string matching two categories' regexes is
  documented in a `KNOWN_OVERLAPS` table with the winner named.
- Keep `category_for_node_kind` unchanged (node-kind fallthrough path).

**Canonical target precedence (the contract this milestone establishes):**
The `CALL_DISPATCH` registry resolves `call_expression`/`call` nodes in this fixed
order; **first match wins**. At M34 only P1/P8/P9 exist (the three current
categories); P2–P7, P10, P11 are slotted in by their respective milestones. Each
later milestone MUST insert at the position below — not append.

```
P1  async_patterns        \.(then|catch|finally)\(                          [existing]
P2  testing               ^(describe|it|test|xit|fit|beforeEach|afterEach|  [M42]
                            beforeAll|afterAll|expect)\( | ^(jest|vi)\. |
                            \bt\.(Run|Error|Fatal|...)\( | \bself\.assert[A-Z]
P3  serialization         ^JSON\.(parse|stringify)\( | ^structuredClone\( |  [M43]
                            ^(json|pickle)\.(loads|dumps|load|dump)\( |
                            ^json\.(Marshal|Unmarshal)\(
P4  schema_validation     ^(z|yup|v|s)\.\w | \.safeParse\( | \bBaseModel\b | [M38]
                            \bField\(
P5  state_store           redux/zustand/jotai/mobx/signal/solid factories +  [M39]
                            ^use(Selector|Dispatch|Store)\b
P6  framework_hooks       ^use[A-Z]                                          [M35]
P7  http_routing          ^(app|router|fastify|server|srv)\.(get|post|...)\( [M41]
                            | Go/Python receiver.verb forms
P8  logging               ^(console|logger|log)\. (TS/JS) + per-lang tables  [existing]
P9  data_access           ^(fetch|axios)\b | \b(query|read|write|get|post|   [existing]
                            put|delete|patch)\( | \b(db|sql)\. | ...
P10 collection_pipelines  \.(map|filter|reduce|flatMap|forEach|find|some|    [M40]
                            every|flat)\(
P11 concurrency           ^Promise\.(all|allSettled|race|any)\( |            [M44]
                            ^asyncio\.(gather|create_task|wait)\(
```

**`KNOWN_OVERLAPS` (documented winners — the disjointness test asserts these):**
1. `useSelector` / `useDispatch` / `useStore` → **state_store** (P5) beats
   `framework_hooks` (P6). More specific wins.
2. `app.get` / `router.post` / `fastify.route` → **http_routing** (P7) beats
   `data_access` (P9). Client fetches (`axios.get`, `fetch`) stay `data_access` —
   the http_routing receiver allowlist (`app|router|fastify|server|srv`) excludes them.
3. `Promise.all([...]).then(cb)` (one node, text contains both `^Promise.all` and
   `.then(`) → **async_patterns** (P1) wins for the outer chained node; the inner
   bare `Promise.all([...])` node (no `.then`) resolves to `concurrency` (P11).
   Document that the combined-text outer node is async by design.

All other category pairs are designed to be **disjoint** per language; the
disjointness test fails on any undocumented overlap.

**Migration Impact:** **None.** Pure refactor. The three existing categories
resolve identically. `tests/m33_sentinels.rs` and the snapshot-identity gate must
stay green with zero diff. `snapshot_version` stays `"1.0"`.

**Files to modify:**
- **Modify:** `crates/sdivi-patterns/src/queries/mod.rs` — registry + dispatch loop.
- **Create:** `crates/sdivi-patterns/tests/dispatch_disjointness.rs`.
- **Modify:** `docs/pattern-categories.md` — formalize precedence + overlap policy.
- **Modify:** `CHANGELOG.md` — under Changed (internal; no behavioural change).

**Acceptance criteria:**
- `cargo test --workspace` green; `m33_sentinels.rs` unchanged and passing.
- Re-running `sdivi snapshot` on a fixture pre/post-refactor yields **bit-identical** JSON.
- `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` pass.

**Tests:**
- Unit: each of the three existing categories resolves through the registry as before.
- Integration: disjointness corpus passes; an intentionally-overlapping string fails
  unless registered in `KNOWN_OVERLAPS`.
- Snapshot-identity: fixture snapshot diff is empty across the refactor.

**Watch For:**
- **Function-pointer table vs trait objects.** Use `fn` pointers, not boxed
  closures — keeps it `const`-friendly and WASM-clean. No allocation.
- **Precedence is the contract now.** Adding a category in M35+ means inserting it
  at the correct precedence position and updating `KNOWN_OVERLAPS`, not appending blindly.
- **`macro_invocation` arm is out of scope** — leave the Rust macro logging/resource
  split exactly as M33 left it.

**Seeds Forward:**
- If `Vec<&str>` multi-category returns ever become real (e.g. `console.error(err)`
  as both `logging` and `error_handling`), the registry generalizes from "first
  match wins" to "collect all matches" with a one-line change. M34 keeps the
  single-winner semantics but makes the generalization trivial.
