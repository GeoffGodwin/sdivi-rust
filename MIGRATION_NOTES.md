# Migration notes

Breaking-change migration guidance for sdivi-rust adopters. Each `0.x → 0.(x+1)`
bump that touches stable surface gets an entry. Post-1.0, the same applies to
major-version bumps.

For the broader migration story from the Python POC
([`structural-divergence-indexer`](https://github.com/GeoffGodwin/structural-divergence-indexer)),
see [`docs/migrating-from-the-python-poc.md`](docs/migrating-from-the-python-poc.md).

## M46 — New `comprehensions` pattern category (Python-only)

**Schema:** unchanged. `snapshot_version` remains `"1.0"`. `list_categories()` count
increases from 18 to 19.

**Config:** unchanged. No new keys. The new category name `comprehensions` is now valid
in `[thresholds.overrides.comprehensions]` blocks.

**What changed.** Four Python node kinds already collected by the Python adapter but
previously dropped are now routed to the new `comprehensions` category:

- `list_comprehension` — `[x for x in xs]`
- `set_comprehension` — `{x for x in xs}`
- `dictionary_comprehension` — `{k: v for k, v in items}`
- `generator_expression` — `(x for x in xs)`, `sum(x*x for x in xs)`

No parsing-layer change. Python-only — other languages produce zero instances.

**Count semantics.** One instance per comprehension node. Nested comprehensions each
emit their own node: `[[x for x in row] for row in matrix]` counts the inner
`list_comprehension` separately from the outer one (two instances total).

**Count-introduction event.** On the first post-M46 snapshot of a Python repo,
`comprehensions` gains a non-zero count (previously dropped). No existing category
loses instances — additive only.

**Escape hatch.** To suppress the new bucket during a migration window:

```toml
[thresholds.overrides.comprehensions]
pattern_entropy_rate = 999.0
expires = "2027-01-01"
reason = "Comprehension category new in M46; accepting baseline divergence."
```

## M45.2 — `error_handling` enriched: Python `except_clause`, Java `catch_clause`/`throw_statement`

**Schema:** unchanged. `snapshot_version` remains `"1.0"`. No new categories — additive
node kinds within an existing category. `list_categories()` count stays 18.

**Config:** unchanged. No new keys.

**What changed.** Three node kinds already collected by their language adapters but
previously dropped are now routed to `error_handling`:

- **Python (node kind):** `except_clause` — individual `except` arms (`except ValueError:`,
  `except (A, B) as e:`). Already emitted by the Python adapter.
- **Java (node kind):** `catch_clause` — individual `catch` arms
  (`catch (IOException e) { ... }`). Already emitted by the Java adapter.
- **Java (node kind):** `throw_statement` — throw sites (`throw new RuntimeException(msg)`).
  Already emitted by the Java adapter.

No parsing-layer change. No new category.

**Double-count semantic.** A Python `try` with 3 `except` arms now yields
1 `try_statement` + 3 `except_clause` = 4 `error_handling` instances. This is
intentional — more arms = more error-flow structure = higher entropy signal. It is
**not** a regression. Rust and TypeScript/JavaScript `error_handling` counts are
unaffected.

**Count-introduction event.** On the first post-M45.2 snapshot of a Python or Java repo,
`error_handling` counts rise. Python gains one count per `except` arm (previously only
the outer `try_statement` counted). Java gains one count per `catch` arm and per `throw`
site. Prior snapshots had no entries for these node kinds — trend continuity is broken
for this dimension at the upgrade boundary. Rust repos are unaffected.

**Escape hatch.** Use a per-category threshold override with an `expires` date:

```toml
[thresholds.overrides.error_handling]
pattern_entropy_rate = 5.0
expires = "2026-12-31"
reason = "Baseline shift from M45.2 adding Python except_clause and Java catch/throw counts"
```

After the `expires` date, default thresholds resume automatically.

## M45.1 — `resource_management` enriched: Python/Go/Java node kinds

**Schema:** unchanged. `snapshot_version` remains `"1.0"`. No new categories — additive
node kinds within an existing category. `list_categories()` count stays 18.

**Config:** unchanged. No new keys.

**What changed.** Three node kinds already collected by their language adapters but
previously dropped are now routed to `resource_management`:

- **Python (node kind):** `with_statement` — context manager blocks (`with open(p) as f:`,
  `with lock:`). Already emitted by the Python adapter.
- **Go (node kind):** `defer_statement` — deferred cleanup (`defer f.Close()`,
  `defer mu.Unlock()`). Already emitted by the Go adapter.
- **Java (node kind):** `try_with_resources_statement` — try-with-resources blocks
  (`try (var r = open(p)) { ... }`). Already emitted by the Java adapter.

No parsing-layer change. No new category.

**Cross-language semantic note.** `with`/`defer`/`try-with-resources` are conceptually
equivalent — all implement scoped acquire → use → release — but are structurally distinct
AST shapes. They share the `resource_management` category intentionally so cross-language
entropy comparisons are uniform. Callers who need per-language breakdowns can filter on
the `node_kind` field in the pattern catalog.

**`defer_statement` is not concurrency.** Go `defer_statement` routes to
`resource_management`, not `concurrency`. The M44 boundary holds: `concurrency` covers
goroutine launches (`go_statement`) and channel multiplexing (`select_statement`) only.

**Count-introduction event.** On the first post-M45.1 snapshot of a Python, Go, or Java
repo, `resource_management` gains non-zero counts from `with_statement`, `defer_statement`,
or `try_with_resources_statement` respectively. Prior snapshots had no entries for these
node kinds — trend continuity is broken for this dimension at the upgrade boundary. Rust
repos are unaffected (Rust `macro_invocation` behaviour is unchanged).

**Escape hatch.** Use a per-category threshold override with an `expires` date:

```toml
[thresholds.overrides.resource_management]
pattern_entropy_rate = 5.0
expires = "2026-12-31"
reason = "Baseline shift from M45.1 adding Python with_statement and Go defer counts"
```

After the `expires` date, default thresholds resume automatically.

## M44 — `concurrency` pattern category introduced

**Schema:** unchanged. `snapshot_version` remains `"1.0"`. `PatternCatalog` JSON shape,
`pattern_metrics` field names, and `DivergenceSummary` structure are all unchanged.

**Config:** unchanged. No new keys.

**What changed.** `concurrency` is a new CALL_DISPATCH category (slot P11, lowest) and
node-kind category. It detects concurrent-execution primitives via two paths:

- **Go (node kind — zero parsing change):** `go_statement` (goroutine launch) and
  `select_statement` (channel multiplexing). These were already collected by the Go adapter
  but previously dropped. They now route to `concurrency`.
- **TypeScript / JavaScript (CALL_DISPATCH P11):** `Promise.all(…)`, `Promise.allSettled(…)`,
  `Promise.race(…)`, `Promise.any(…)`.
- **Python (CALL_DISPATCH P11):** `asyncio.gather(…)`, `asyncio.create_task(…)`,
  `asyncio.wait(…)`, `asyncio.as_completed(…)`, `asyncio.run(…)`.

`list_categories()` count grows from 17 → 18.

**Boundary with `async_patterns`.** `async_patterns` covers single-future `.await` and
`.then/catch/finally` chaining; `concurrency` covers multi-future coordination
(`Promise.all`) and goroutine/channel primitives. The two categories are disjoint for
bare calls. A chained `Promise.all([]).then(cb)` AST node matches both P1 (`.then(`) and
P11; `async_patterns` wins by precedence.

**`defer_statement` is not concurrency.** It belongs to `resource_management` (M45.1).

**Disjointness.** `Promise.all` does not match `async_patterns` (which requires `.then(`,
`.catch(`, or `.finally(`). `asyncio.gather` does not match Python `data_access`
(`^(open\(|requests\.|...)`) or `logging` (`^(logging\.|print\b)`). No new `KNOWN_OVERLAPS`
entries are required.

**Count-introduction event.** On the first post-M44 snapshot of a Go repo with goroutines
or select blocks, `concurrency` transitions from zero to non-zero. Prior snapshots had no
`concurrency` bucket — trend continuity is broken for this dimension at the upgrade boundary.
The trend line resumes cleanly from the second post-upgrade snapshot.

**Escape hatch.** Set per-category threshold overrides with an `expires` date:

```toml
[thresholds.overrides.concurrency]
pattern_entropy_rate = 5.0
expires = "2026-12-31"
reason = "Migrating Go services to structured concurrency; goroutine count in flux"
```

After the `expires` date, default thresholds resume automatically.

## M43 — `serialization` pattern category introduced

**Schema:** unchanged. `snapshot_version` remains `"1.0"`. `PatternCatalog` JSON shape,
`pattern_metrics` field names, and `DivergenceSummary` structure are all unchanged.

**Config:** unchanged. No new keys.

**What changed.** `serialization` is now a native CALL_DISPATCH category (slot P3, below
`testing` P2 and above `schema_validation` P4), classified via receiver-anchored callee-text:

- **TypeScript / JavaScript:** `JSON.parse(…)`, `JSON.stringify(…)`, `structuredClone(…)`.
- **Python:** `json.loads(…)`, `json.dumps(…)`, `json.load(…)`, `json.dump(…)`,
  `pickle.loads(…)`, `pickle.dumps(…)`.
- **Go:** `json.Marshal(…)`, `json.Unmarshal(…)`, `json.MarshalIndent(…)`,
  `json.NewEncoder(…)`, `json.NewDecoder(…)`.

`list_categories()` count grows from 16 → 17.

**Disjointness.** The `json.` receiver is not in the Python `data_access` regex
(`^(open\(|requests\.|httpx\.|cursor\.|session\.|conn\.)`), so `json.loads(s)` routes to
`serialization`, not `data_access`. Bare `.parse(` is intentionally excluded — it stays in
`schema_validation` (`UserSchema.safeParse(x)`) or falls through unmatched. No existing callee
strings change category. No `KNOWN_OVERLAPS` entries added.

**Count-introduction event.** On the first post-M43 snapshot of a repo whose source files
include serialization calls, `serialization` transitions from zero to non-zero. Prior snapshots
had no `serialization` bucket — trend continuity is broken for this dimension at the upgrade
boundary. The trend line resumes cleanly from the second post-upgrade snapshot.

**Escape hatch.** Set per-category threshold overrides with an `expires` date:

```toml
[thresholds.overrides.serialization]
pattern_entropy_rate = 5.0
expires = "2026-12-31"
reason = "Migrating from JSON.parse to typed Zod parsers; expect churn during migration"
```

After the `expires` date, default thresholds resume automatically.

## M42 — `testing` pattern category introduced

**Schema:** unchanged. `snapshot_version` remains `"1.0"`. `PatternCatalog` JSON shape,
`pattern_metrics` field names, and `DivergenceSummary` structure are all unchanged.

**Config:** unchanged. No new keys.

**What changed.** `testing` is now a native CALL_DISPATCH category (slot P2, just below
`async_patterns` P1 and above all other categories), classified via callee-text inspection:

- **TypeScript / JavaScript** (Jest/Vitest/Mocha/Jasmine): BDD globals (`describe`, `it`,
  `test`, `context`), lifecycle hooks (`beforeEach`, `afterEach`, `beforeAll`, `afterAll`),
  `expect(…)` assertion roots, focused/excluded variants (`xit`, `xdescribe`, `fit`,
  `fdescribe`), and framework-namespaced helpers (`jest.fn`, `jest.mock`, `jest.spyOn`,
  `vi.fn`, `vi.mock`, `vi.spyOn`, `jest.clearAllMocks`, `jest.useFakeTimers`, etc.).
- **Go:** `testing.T` method calls — `t.Run`, `t.Error`, `t.Errorf`, `t.Fatal`,
  `t.Fatalf`, `t.Helper`, `t.Skip`, `t.Skipf`, `t.Log`, `t.Logf`, `t.Cleanup`,
  `t.Parallel`. Matched via `\bt\.` word-boundary anchor on the receiver.
- **Python:** `unittest.TestCase` assertion methods — `self.assertEqual`,
  `self.assertTrue`, and the full `self.assert[A-Z]…` family.

`list_categories()` count grows from 15 → 16.

**Count-introduction event.** On the first post-M42 snapshot of a repo whose test files
are included in the pattern scope, `testing` transitions from zero to non-zero. Prior
snapshots had no `testing` bucket — trend continuity is broken for this dimension at the
upgrade boundary. The trend line resumes cleanly from the second post-upgrade snapshot.

**`scope_exclude` interaction.** `patterns.scope_exclude` removes files from the pattern
catalog only (they stay in the dependency graph). The `testing` bucket is non-empty only
when test files are included in the pattern scope. If a repo excludes test paths:

```toml
[patterns]
scope_exclude = ["**/*.test.ts", "**/*.spec.ts", "**/tests/**", "**/test_*.py"]
```

the `testing` bucket will be empty on all snapshots — not a bug, but the expected
behaviour for teams that want to measure production-code patterns only. No auto-detection
of test paths is performed.

**Known false positives.** `test(args)`, `it(args)`, `context(args)`, and `expect(x)` are
valid identifiers in business logic. The `^` anchor prevents mid-identifier matching but
cannot distinguish intent. At codebase scale these are entropy noise; they do not
materially affect divergence scores in repos where test files dwarf non-test usage.

**Escape hatch.** Set per-category threshold overrides with an `expires` date:

```toml
[thresholds.overrides.testing]
pattern_entropy_rate = 5.0
expires = "2026-12-31"
reason = "Migrating test suite from Mocha to Vitest; expect churn during migration"
```

After the `expires` date, default thresholds resume automatically.

## M41 — `http_routing` pattern category introduced; `app.get`/`router.post` reassigned from `data_access`

**Schema:** unchanged. `snapshot_version` remains `"1.0"`. `PatternCatalog` JSON shape,
`pattern_metrics` field names, and `DivergenceSummary` structure are all unchanged.

**Config:** unchanged. No new keys.

**What changed.** `http_routing` is now a native CALL_DISPATCH category (slot P7, above
`logging` P8 and `data_access` P9), classified via receiver-allowlist-anchored callee-text:

- **TypeScript / JavaScript** (Express/Koa/Fastify/Hono): receiver `app`, `router`, `fastify`,
  `server`, or `srv` + method `get|post|put|delete|patch|head|options|all|use|route`.
- **Go** (net/http, Gin, Echo, Gorilla Mux): receiver `http`, `mux`, `r`, `e`, `router`,
  `engine`, `g`, or `rg` + method `HandleFunc|Handle|GET|POST|PUT|DELETE|PATCH|Any|Group`.
- **Python** (Flask/FastAPI imperative): `*.add_url_rule(` member-call pattern.

`list_categories()` count grows from 14 → 15.

**Headline migration: `app.get` / `router.post` now resolve to `http_routing`.**
This is the most consequential change for TypeScript/JavaScript backends.

Before (≤M40):
```
call_expression "app.get('/users', listUsers)"  →  data_access
call_expression "router.post('/user', createUser)"  →  data_access
```

After (M41+):
```
call_expression "app.get('/users', listUsers)"  →  http_routing
call_expression "router.post('/user', createUser)"  →  http_routing
```

`data_access` counts decrease; `http_routing` counts are a new non-zero bucket.
On the first post-M41 snapshot, per-file counts for server route registrations shift
from `data_access` to `http_routing`. Trend continuity is broken for `data_access` at
the upgrade boundary; the trend line resumes from the second post-upgrade snapshot.

**Client calls are unaffected.** `axios.get(url)`, `fetch(url)`, `client.get(url)`, and
`cache.get(key)` stay in `data_access` — their receiver (`axios`, `client`, `cache`) is
outside the `http_routing` allowlist.

**Decorator-style routes are unaffected.** NestJS (`@Get('/')`, `@Post(...)`) and FastAPI
(`@app.get(...)`) route declarations are `decorator`/`decorated_definition` nodes, already
classified under `decorators` (M36.1/M36.2). They are counted once — in `decorators` — and
are not reassigned here.

**Idiosyncrasy gap.** A server variable with an unrecognised name (`const api = express();
api.get(...)`) will not be classified as `http_routing` — receiver-type inference is outside
the v0 node-kind model. Such calls continue to resolve via the data_access verb-list regex.
Document this as a known limitation; no configuration escape hatch is available.

**Escape hatch.** Set per-category threshold overrides with an `expires` date:

```toml
[thresholds.overrides.http_routing]
pattern_entropy_rate = 5.0
expires = "2026-12-31"
reason = "Migrating imperative app.get routes to router pattern"
```

After the `expires` date, default thresholds resume automatically.

## M40 — `collection_pipelines` pattern category introduced

**Schema:** unchanged. `snapshot_version` remains `"1.0"`. `PatternCatalog` JSON shape,
`pattern_metrics` field names, and `DivergenceSummary` structure are all unchanged.

**Config:** unchanged. No new keys.

**What changed.** `collection_pipelines` is now a native CALL_DISPATCH category (slot P10,
lowest-priority member-call category — all more specific categories resolve first),
classified via member-call callee-text inspection in TypeScript, JavaScript, Go, and Java:

- `.map(`, `.filter(`, `.reduce(`, `.flatMap(`, `.forEach(`, `.find(`, `.findIndex(`,
  `.some(`, `.every(`, `.flat(` on any receiver are classified here.

`list_categories()` count grows from 13 → 14.

**Count-introduction event.** On the first post-M40 snapshot of a TypeScript or JavaScript
repo that uses standard array/iterable pipeline methods, `collection_pipelines` transitions
from zero to non-zero. Prior snapshots had no `collection_pipelines` bucket — trend
continuity is broken for this dimension at the upgrade boundary. The trend line resumes
cleanly from the second post-upgrade snapshot onward.

**Receiver-type noise.** Callee-text cannot distinguish an array `.map` from
`rxObservable.map(fn)` (RxJS), `new Map().forEach(cb)` (ES6 Map), or
`domNodeList.forEach(cb)` (DOM NodeList). All match. This is intentional — the signal
is the functional-iteration population at codebase scale (entropy / convention drift),
not the receiver type of individual calls. No action required unless you need precise
receiver-type discrimination (which would require a type-info pass outside the v0 model).

**Disjoint from `data_access` and `async_patterns`.** The method-name sets are
non-overlapping: `data_access` uses `query/read/write/fetch`; `async_patterns` uses
`then/catch/finally`; none appear in `collection_pipelines`.

**Escape hatch.** Set per-category threshold overrides with an `expires` date:

```toml
[thresholds.overrides.collection_pipelines]
pattern_entropy_rate = 5.0
expires = "2026-12-31"
reason = "Migrating imperative loops to functional pipelines"
```

After the `expires` date, default thresholds resume automatically.

## M39 — `state_store` pattern category introduced; `useSelector`/`useDispatch`/`useStore` precedence reassignment

**Schema:** unchanged. `snapshot_version` remains `"1.0"`. `PatternCatalog` JSON shape,
`pattern_metrics` field names, and `DivergenceSummary` structure are all unchanged.

**Config:** unchanged. No new keys.

**What changed.** `state_store` is now a native CALL_DISPATCH category (slot P5, above
`framework_hooks` P6), classified via callee-text inspection in TypeScript and JavaScript:

- **Redux / RTK:** `createSlice`, `configureStore`, `createStore`, `combineReducers`,
  `createAsyncThunk`, `createReducer`, `createAction`.
- **React-Redux hooks:** `useSelector`, `useDispatch`, `useStore`.
- **Zustand:** bare `create(` (anchored at callee start — `prisma.user.create(data)` is excluded).
- **Jotai / Recoil:** `atom`, `selector`, `atomFamily`, `selectorFamily`.
- **MobX:** `observable`, `action`, `computed`, `makeObservable`, `makeAutoObservable`, `runInAction`.
- **Signals (Preact/Angular):** `signal`, `computed`, `effect`, `batch`.
- **Solid:** `createSignal`, `createEffect`, `createMemo`, `createStore`, `createResource`.

`list_categories()` count grows from 12 → 13.

**Precedence reassignment (the canonical example).** `useSelector`, `useDispatch`, and
`useStore` previously resolved to `framework_hooks` (P6) via the `^use[A-Z]` regex.
They now resolve to `state_store` (P5) via the more specific `^use(Selector|Dispatch|Store)\b`
regex, which wins by CALL_DISPATCH first-match precedence. **Effect:** on the first
post-M39 snapshot, per-file counts for these three hooks shift from the `framework_hooks`
bucket to `state_store`. The total hook count across both buckets is unchanged; the
distribution changes. This is intentional — Redux hook calls are state-management calls,
not generic component-composition hooks.

**`^`-anchor rationale.** All factory patterns are anchored at callee start. Bare imported
calls (`create(...)`, `atom(0)`) match; member-access ORM/DOM calls
(`prisma.user.create(data)`, `document.createElement('div')`) do not.

**Open question (TanStack Query / SWR).** `useQuery`, `useMutation`, `useSWR` blur "state"
and "data-fetching". Until a follow-up decides their home, they fall through to
`framework_hooks` unchanged — no count shift for these hooks in M39.

**Escape hatch.** Set per-category threshold overrides with an `expires` date:

```toml
[thresholds.overrides.state_store]
pattern_entropy_rate = 5.0
expires = "2027-06-30"
reason = "M39 upgrade: state_store newly populated; setting initial tolerance"
```

**Trend continuity.** The first post-M39 snapshot transitions `state_store` from zero to
non-zero (count-introduction event). `framework_hooks` may see a count reduction if the
repo uses `useSelector`/`useDispatch`/`useStore`. Subsequent snapshots establish the new
baseline. The delta for both transitions is not meaningful as a drift signal.

## M38 — `schema_validation` pattern category introduced (TS/JS/Python count-introduction event)

**Schema:** unchanged. `snapshot_version` remains `"1.0"`. `PatternCatalog` JSON shape,
`pattern_metrics` field names, and `DivergenceSummary` structure are all unchanged.

**Config:** unchanged. No new keys.

**What changed.** `schema_validation` is now a native CALL_DISPATCH category (slot P4),
classified via callee-text inspection in TypeScript, JavaScript, and Python:

- **TypeScript / JavaScript:** Zod (`z.object`, `z.string`, `z.enum`), Yup (`yup.object()`,
  `yup.string()`), Valibot (`v.object`, `v.pipe`), Superstruct (`s.object`) — detected via the
  namespace-anchored regex `^(z|yup|v|s)\.\w`. Additionally `.safeParse(` is matched as a
  Zod-specific validated-parse call.
- **Python:** Pydantic field-constraint calls — `Field(...)`, `constr(...)`, `conint(...)` —
  detected via `\bField\(|\bconstr\(|\bconint\(`. Note: `class Foo(BaseModel)` is a
  `class_definition` counted under `class_hierarchy`, not here. Python coverage is
  intentionally partial in v0.

`list_categories()` count grows from 11 → 12.

**Precision over recall.** The TS/JS regex is namespace-anchored (`z.`/`yup.`/`v.`/`s.`),
not method-name-anchored. `SomeSchema.parse(x)` where the receiver name is arbitrary is a
known miss — receiver-type info is outside the v0 model. Bare `.string()`/`.object()` on
arbitrary receivers are intentionally excluded to avoid flooding the bucket.

**Cross-category note.** class-validator decorators (`@IsString()`, `@IsEmail()`) belong to
`decorators` (M36.1/M36.2), not `schema_validation`. The split is intentional — decorator-shape
entropy and schema-declaration entropy are independent signals. See `docs/pattern-categories.md`
for the rationale.

**Escape hatch.** Set per-category threshold overrides with an `expires` date:

```toml
[thresholds.overrides.schema_validation]
pattern_entropy_rate = 5.0
expires = "2027-06-30"
reason = "M38 upgrade: schema_validation bucket newly populated; setting initial tolerance"
```

**Trend continuity.** The first post-M38 snapshot transitions `schema_validation` from zero to
non-zero. This is a count-introduction event — the same class as M35 (`framework_hooks`), M36.1
(`decorators`), and M37 (`null_safety`). The delta for this transition is not meaningful as a
drift signal; subsequent snapshots establish the baseline.

## M37 — `null_safety` pattern category introduced (TS/JS count-introduction event)

**Schema:** unchanged. `snapshot_version` remains `"1.0"`. `PatternCatalog` JSON shape,
`pattern_metrics` field names, and `DivergenceSummary` structure are all unchanged.

**Config:** unchanged. No new keys.

**What changed.** Two tree-sitter node kinds are now collected by the TypeScript and
JavaScript language adapters and classified as `null_safety`:

- `optional_chain` — optional chaining (`a?.b`, `arr?.[0]`) in both TS and JS.
  Note: optional calls (`fn?.()`) emit `call_expression` in the grammar, not
  `optional_chain` — they are not classified under `null_safety`.
- `non_null_expression` — TypeScript non-null assertion operator (`el!`); TS only.

`list_categories()` count grows from 10 → 11.

**Count semantics.** Each `optional_chain` node emitted by the grammar counts as one
instance. A long chain `a?.b?.c` may produce nested `optional_chain` nodes — each
counts independently. This per-node counting is deterministic.

**Deferred: nullish coalescing (`??`).** `a ?? b` is a `binary_expression` with a `??`
operator child, not a dedicated node kind. Operator-field inspection is out of scope for
the v0 node-kind model. `binary_expression` is intentionally excluded.

**Escape hatch.** Set per-category threshold overrides with an `expires` date:

```toml
[thresholds.overrides.null_safety]
pattern_entropy_rate = 5.0
expires = "2027-06-30"
reason = "M37 upgrade: null_safety bucket newly populated; setting initial tolerance"
```

**Trend continuity.** The first post-M37 snapshot transitions `null_safety` from zero
to non-zero. This is a count-introduction event — the same class as M35 (`framework_hooks`)
and M36.1 (`decorators`). The delta for this transition is not meaningful as a drift
signal; subsequent snapshots establish the baseline.

## M33 — Native pipeline switchover to `classify_hint` (per-category instance counts shift)

**Schema:** unchanged. `snapshot_version` remains `"1.0"`. `PatternCatalog` JSON shape,
`pattern_metrics` field names, and `DivergenceSummary` structure are all unchanged.

**Config:** unchanged. No new keys. Existing `[thresholds.overrides.<category>]` blocks
remain the escape hatch — set `expires` to a date within your migration window to defer
recalibration.

**What changed.** `crates/sdivi-patterns/src/catalog.rs` now calls `classify_hint`
(callee-text-aware) instead of `category_for_node_kind` (node-kind-only). The result:

1. **`data_access` shrinks.** Pre-M33 every `call_expression`/`call` was a `data_access`
   instance. Post-M33 only callees matching the per-language regex are. On a typical
   TS/JS codebase this drops `data_access` instance count substantially.
2. **`logging` becomes non-zero.** Was catalog-only (zero natively) since M30. Now natively
   populated. `console.log`, `tracing::info!`, `fmt.Println` etc. flow here.
3. **`async_patterns` grows on TS/JS.** Promise chains (`.then()/.catch()/.finally()`) are
   now classified as `async_patterns` instead of being dropped or going to `data_access`.
4. **`resource_management` shrinks on Rust.** Logging macros leave the bucket; only
   non-logging macros (`vec!`, `assert!`, `format!`, etc.) remain.
5. **Threshold gates may trip.** The M20 epsilon for cross-architecture float drift is far
   smaller than the M33 instance-count shifts — adopters should not expect the epsilon to
   absorb the change.

**Escape hatch.** Set per-category threshold overrides with an `expires` date:

```toml
[thresholds.overrides.data_access]
pattern_entropy_rate = 5.0
expires = "2026-09-30"
reason = "M33 upgrade: data_access count dropped; recalibrating baseline"

[thresholds.overrides.logging]
pattern_entropy_rate = 5.0
expires = "2026-09-30"
reason = "M33 upgrade: logging bucket newly populated; setting initial tolerance"
```

**Worked example — `simple-typescript` fixture, pre-M33 vs post-M33.**

Pre-M33 `pattern_metrics` (all `call_expression` → `data_access`):

```json
{
  "pattern_metrics": {
    "data_access": { "entropy": 0.0, "instance_count": 2, "convention_drift": 0.0 },
    "async_patterns": { "entropy": 0.0, "instance_count": 1, "convention_drift": 0.0 }
  }
}
```

Post-M33 `pattern_metrics` (after extending fixture with `console.log` and `fetch`):

```json
{
  "pattern_metrics": {
    "logging":      { "entropy": 0.0, "instance_count": 1, "convention_drift": 0.0 },
    "data_access":  { "entropy": 0.0, "instance_count": 1, "convention_drift": 0.0 },
    "async_patterns": { "entropy": 0.0, "instance_count": 1, "convention_drift": 0.0 }
  }
}
```

Key differences visible in a `compute_delta` between pre- and post-M33 snapshots:
- `data_access.instance_count` drops (was counting `helper(...)`, `path.replace(...)`;
  now only `fetch(...)` matches).
- `logging` key appears for the first time.
- `async_patterns` key may appear or grow (`.then()`/`.catch()` Promise chains).

**Foreign extractors are unaffected.** Embedders that supply `PatternInstanceInput`
directly bypass `build_catalog` entirely — their inputs determine their outputs.
If you have already migrated to `classify_hint` in M32, you are now aligned with the
native pipeline. If not, your hand-rolled filter continues to work unchanged.

## M36.2 — `decorators` category extended to Python (`decorated_definition` added)

**Schema:** unchanged. `snapshot_version` remains `"1.0"`.

**What changed.** `decorated_definition` (tree-sitter-python's wrapper node for
`@`-decorated function and class definitions) is now included in
`decorators::NODE_KINDS`. The Python adapter already emitted `decorated_definition`
hints; they were previously uncollected by any category. After upgrade, Python
repositories that use `@dataclass`, `@property`, `@app.route(...)`,
`@pytest.fixture`, `@app.task`, `@cached_property`, etc. gain a non-zero
`decorators` bucket on the next snapshot.

**Count semantics (Python vs. TypeScript/JavaScript).**

- TypeScript/JavaScript (M36.1): one instance **per decorator** (`decorator` node).
  Three stacked `@`-lines on one class = three instances.
- Python (M36.2): one instance **per decorated function or class**
  (`decorated_definition` wrapper). Three stacked `@`-lines on one function =
  **one** instance.

This cross-language asymmetry is an intentional v0 simplification documented in
`docs/pattern-categories.md`. Cross-language comparison of raw `decorators` counts
must account for this difference. Aligning granularity (making Python also emit
per-decorator counts) is deferred until a concrete cross-language comparison
consumer needs symmetric counts.

**`list_categories()` count:** unchanged (10). No new category; Python repos gain
counts in the existing `decorators` bucket.

**Escape hatch.** Use a threshold override to defer recalibration:

```toml
[thresholds.overrides.decorators]
pattern_entropy_rate = 5.0
expires = "2026-12-31"
reason = "M36.2 upgrade: decorators bucket now includes Python decorated_definition; recalibrating baseline"
```

**Trend continuity.** `snapshot_version` stays `"1.0"`. The `decorators` bucket
was already present since M36.1 — the bucket grows on the first post-M36.2 snapshot
for Python repos; `compute_delta` reports a count-introduction delta.

## M36.1 — `decorators` category introduction (TS/JS decorator count appears)

**Schema:** unchanged. `snapshot_version` remains `"1.0"`.

**What changed.** The `decorators` pattern category is now natively classified for
TypeScript and JavaScript. The parsing stage (`sdivi-lang-typescript` and
`sdivi-lang-javascript`) now emits `decorator` nodes as `PatternHint` values —
previously the `decorator` node kind was not collected at all. `category_for_node_kind`
routes `"decorator"` to `"decorators"` in the native pipeline.

**Impact on existing snapshots.**

1. **`decorators` transitions from zero to non-zero.** On TS/JS repos using decorator
   syntax (NestJS, Angular, TypeORM, MikroORM, class-validator, etc.), the `decorators`
   bucket was absent in all pre-M36.1 snapshots. After upgrade, the first snapshot
   counts all decorator nodes. `compute_delta` will report a large positive delta on
   `decorators.instance_count` — expected; not a regression.
2. **No existing category loses instances.** `decorator` was previously uncollected;
   no prior category is cannibalised.
3. **`list_categories()` count grows from 9 → 10.** Embedders that hard-code the
   count must update. The recommended pattern is `list_categories().categories.len()`.
4. **Parsing-layer change:** the parsing stage now emits more `PatternHint` values per
   file on TS/JS repos with decorators. Snapshot `feature_record` hint counts will
   increase on decorator-heavy files.

**Escape hatch.** Use a threshold override to defer recalibration:

```toml
[thresholds.overrides.decorators]
pattern_entropy_rate = 5.0
expires = "2026-12-31"
reason = "M36.1 upgrade: decorators bucket newly populated; setting initial tolerance"
```

**Trend continuity.** `snapshot_version` stays `"1.0"`. The first post-M36.1 snapshot
is comparable to prior snapshots — `compute_delta` returns `null` for `decorators`
when no prior `decorators` value exists, and a numeric delta on subsequent comparisons.

## M35 — `framework_hooks` category introduction (TS/JS hook-call count appears)

**Schema:** unchanged. `snapshot_version` remains `"1.0"`.

**What changed.** The `framework_hooks` pattern category is now natively classified
for TypeScript and JavaScript via `classify_hint` callee-text inspection. Any
`call_expression` callee matching `^use[A-Z]` (`useState`, `useEffect`, `useMemo`,
custom hooks like `useAuth`, etc.) is routed to `framework_hooks`.

**Impact on existing snapshots.**

1. **`framework_hooks` transitions from zero to non-zero.** On TS/JS repos, the
   `framework_hooks` bucket was absent (or zero) in all pre-M35 snapshots. After
   upgrade, the first snapshot counts all hook calls. `compute_delta` will report
   a large positive delta on `framework_hooks.instance_count` — expected; not a
   regression.
2. **No existing category loses instances.** Hook callees (`useState`, etc.) did
   not match any prior regex (they were dropped as unrecognised). `data_access`,
   `logging`, and `async_patterns` are unaffected.
3. **`list_categories()` count grows from 8 → 9.** Embedders that hard-code the
   count must update. The recommended pattern is `list_categories().categories.len()`.

**Escape hatch.** Use a threshold override to defer recalibration:

```toml
[thresholds.overrides.framework_hooks]
pattern_entropy_rate = 5.0
expires = "2026-12-31"
reason = "M35 upgrade: framework_hooks bucket newly populated; setting initial tolerance"
```

**Trend continuity.** `snapshot_version` stays `"1.0"`. The first post-M35 snapshot
is comparable to prior snapshots — `compute_delta` returns `null` for
`framework_hooks` when no prior `framework_hooks` value exists, and a numeric delta
on subsequent comparisons.

## M28 — Leiden performance (modularity values may shift slightly)

**Schema:** unchanged. `snapshot_version` remains `"1.0"`. `LeidenPartition` JSON shape is unchanged. `BoundarySpec` YAML untouched.

**Config:** two new optional `[boundaries]` keys (`leiden_min_compression_ratio`, `leiden_max_recursion_depth`). Existing configs without them inherit the defaults transparently via `#[serde(default)]`. No action needed.

**Modularity values:** snapshots taken post-M28 may report modularity values that differ by up to ~1% from snapshots taken pre-M28 on the same repo state. This is within the `verify-leiden` 1% tolerance band — the same situation that arose with M18. The algorithm is mathematically equivalent; the numerical difference stems from the new compression-ratio cutoff stopping recursion a level earlier on sparse graphs. If you are running trend analysis that compares pre-M28 and post-M28 snapshots, treat the M28 cutover as a baseline reset for modularity-sensitive metrics.

**WASM:** `WasmLeidenConfigInput` gains two optional TypeScript fields (`min_compression_ratio?: number`, `max_recursion_depth?: number`). Existing callers that omit these fields continue to work — they receive the defaults (0.1 and 32).

## 0.2.x → 0.3.0 (M25 + M26 resolver fixes — no schema break)

### Graph resolver: parent navigation and per-language dispatch (M26)

**What changed.** The graph resolver previously stripped `../` and `super::`
characters but never walked up the directory tree, so all parent-relative
imports resolved to nothing. It also had no per-language dispatch — Python
dotted specifiers, Go module-path imports, and Java dotted class names were all
dropped silently.

M26 fixes this:
- `../` and `../../` imports now walk up the correct number of directory levels
  before resolving file extensions (TypeScript, JavaScript, Python, etc.).
- `super::` Rust imports navigate to the parent directory and search for the
  stem there before falling back to the global stem map.
- Python: `foo.bar` resolves to `foo/bar.py` or `foo/bar/__init__.py`; relative
  imports with leading dots (`.sibling`, `..pkg`) resolve per PEP 328.
- Go: module-path imports strip the `module` prefix from `go.mod` and resolve
  to all `.go` files in the resulting directory. The pipeline reads `go.mod`
  automatically; non-pipeline callers use `build_dependency_graph_with_go_module`.
- Java: `com.acme.lib.Util` resolves via standard Maven source roots
  (`src/main/java`, `src/test/java`) plus dynamically discovered module roots.
  Wildcard imports (`com.acme.lib.*`) emit one edge per class file in the package.

**Schema impact.** None. `snapshot_version` stays `"1.0"`.

**Baseline impact.** Edge counts increase substantially — especially on
Python, Go, and Java projects. The same re-baseline or threshold-override
strategy from M25 applies; see below.

### Import specifier extraction: substantial edge-count increase on non-Rust projects (M25)

**What changed.** Language adapters for Python, TypeScript, JavaScript, Go, and
Java previously emitted whole import-statement text into `FeatureRecord::imports`
(e.g. `"import { foo } from '../lib/x'"`). The graph resolver silently dropped
every such string, producing zero cross-file edges for all five languages.
Adapters now emit only the module specifier (e.g. `"../lib/x"`). Edges that
were previously invisible now resolve, and all coupling-based metrics become
meaningful.

**Schema impact.** None. `snapshot_version` stays `"1.0"`. Pre-M25 snapshots
are still readable; the change affects only the content of future snapshots.

**Baseline impact.** The first `sdivi snapshot` after upgrading will produce
a large `coupling_delta` and `community_count_delta` against any pre-M25
baseline on a Python/TS/JS/Go/Java project. `boundary_violation_rate` will
likely increase if you have a `.sdivi/boundaries.yaml` declared, because
violations that were previously undetectable now appear.

**Recommended migration:**

Option A — re-baseline (cleanest):
```bash
rm .sdivi/snapshots/*.json   # clear old baselines
sdivi snapshot               # first snapshot under new adapter
```

Option B — one-time override (preserves trend history):
```toml
# .sdivi/config.toml — expires after the spike settles
[thresholds.overrides.coupling]
coupling_delta_rate = 50.0
expires = "2026-06-01"
reason = "M25+M26 fixes; first post-upgrade snapshot has large coupling_delta"

[thresholds.overrides.boundaries]
boundary_violation_rate = 20.0
expires = "2026-06-01"
reason = "M25+M26 fixes; first post-upgrade snapshot may spike violations"
```

## 0.1.x

No breaking changes between 0.1.0 and 0.1.14. Every release in the 0.1 line is
backwards-compatible at the public-API and snapshot-schema level. New `Input`
fields are added with `#[serde(default)]` and new snapshot fields are
additive.

The 0.1.7 algorithm correction in the Leiden refinement phase is not a public
API break. It does invalidate trend continuity across the 0.1.6 / 0.1.7
boundary because pre-0.1.7 snapshots have a `modularity` value derived from
the broken refinement. See `CHANGELOG.md` 0.1.7 entry.

## 0.1.x → 0.2.0

### `assemble_snapshot` parameter type change

**What changed.** The fifth positional parameter of
`sdivi_snapshot::assemble_snapshot` (re-exported as
`sdivi_core::assemble_snapshot`) is now `boundary_count: Option<usize>` instead
of `boundary_spec: Option<&sdivi_config::BoundarySpec>`. The function no longer
reaches into a `BoundarySpec` to read `.boundaries.len()`; the caller does that
inline (or supplies the count from any other source).

**Why.** Two reasons:

1. The function only ever read one integer (`spec.boundaries.len()`) from the
   spec; the rest of the type was dead weight in the signature. Asking for the
   bag instead of the integer it contains was unnecessary coupling.
2. WASM and other non-FS callers cannot construct a `BoundarySpec` (it lives in
   `sdivi-config` and is parsed from YAML). The previous WASM binding worked
   around this by calling `assemble_snapshot` with `None` and then mutating
   `snap.intent_divergence` after the fact — a second assembly seam outside the
   canonical function. That seam is gone now.

A side effect: `sdivi-snapshot` no longer depends on `sdivi-config`.

**What to do.** Mechanical replacement at every call site.

```diff
-let snap = assemble_snapshot(
-    graph, partition, catalog, pattern_metrics,
-    boundary_spec.as_ref(),
-    &timestamp, commit, change_coupling, violation_count,
-);
+let boundary_count = boundary_spec.as_ref().map(|spec| spec.boundaries.len());
+let snap = assemble_snapshot(
+    graph, partition, catalog, pattern_metrics,
+    boundary_count,
+    &timestamp, commit, change_coupling, violation_count,
+);
```

Callers that already passed `None` need no change — `None` continues to mean
"omit `intent_divergence` from the snapshot."

**Trend continuity.** Unaffected. Snapshot JSON output is byte-identical for
the same inputs; the change is purely at the Rust API surface. Snapshots
written by 0.1.x can be loaded and diffed by 0.2.0 without conversion.

## Future entries

When a breaking change lands, document:

- **What changed.** A precise description of the renamed, removed, or
  resemanticised item.
- **Why.** The motivation. Often a correctness fix or a SemVer-mandated
  cleanup.
- **What to do.** A concrete migration recipe. A diff or `sed` snippet
  beats prose.
- **Trend continuity.** Whether snapshots from prior versions are still
  comparable.
