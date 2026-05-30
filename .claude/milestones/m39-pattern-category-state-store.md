
#### Milestone 39: Pattern Category — `state_store`

<!-- milestone-meta
id: "39"
status: "planned"
-->

**Scope:** Add the `state_store` category for external/library state-management
idioms, distinct from the existing closure-based `state_management`. Covers Redux /
RTK (`createSlice`, `configureStore`, `useSelector`, `useDispatch`, `connect`),
Zustand (`create(...)`), Jotai (`atom(...)`), Recoil (`atom`, `selector`), MobX
(`observable`, `action`, `makeAutoObservable`), and signals
(`signal`, `computed`, `effect`). Callee-text on `call_expression`; depends on M34
and M35 (hook-overlap precedence). No parsing-layer change.

**Why this milestone exists:** `state_management` (closures/arrow functions) and
library-level state stores are different signals. Teams converge on one store
approach; divergence (mixed Redux + Zustand + Context) is a meaningful, judgment-free
drift indicator that the closure bucket cannot express.

**Deliverables:**
- Create `crates/sdivi-patterns/src/queries/state_store.rs` with TS/JS regex table.
- Register in `ALL_CATEGORIES`, the M34 `CALL_DISPATCH` registry at **slot P5
  (above `framework_hooks` P6)** so `useSelector`/`useDispatch`/`useStore` resolve
  here, not to hooks; record the overlap in `KNOWN_OVERLAPS`. Add `CATALOG_ENTRIES`.
- Update `docs/pattern-categories.md`.

**Detection (finalized — generic factory names `^`-anchored so only bare/imported
calls match, never `obj.create(...)` member calls):**
| Library family | Pattern | Examples matched |
|---|---|---|
| Redux / RTK | `^(createSlice\|configureStore\|createStore\|combineReducers\|createAsyncThunk\|createReducer\|createAction)\(` | `createSlice({...})`, `configureStore({...})` |
| React-Redux hooks | `^use(Selector\|Dispatch\|Store)\b` | `useSelector(s => s.x)`, `useDispatch()` |
| Zustand | `^create\(` | `create((set) => ({...}))` |
| Jotai / Recoil | `^(atom\|selector\|atomFamily\|selectorFamily)\(` | `atom(0)`, `selector({...})` |
| MobX | `^(observable\|action\|computed\|makeObservable\|makeAutoObservable\|runInAction)\(` | `makeAutoObservable(this)`, `observable({...})` |
| Signals (Preact/Angular) | `^(signal\|computed\|effect\|batch)\(` | `signal(0)`, `computed(() => ...)` |
| Solid | `^create(Signal\|Effect\|Memo\|Store\|Resource)\(` | `createSignal(0)`, `createStore({...})` |

**The `^`-anchor is the key precision decision:** imported store factories are
called bare (`create(...)`, `atom(0)`, `signal(0)`), so anchoring at callee start
captures them while excluding member-style calls like `prisma.user.create(...)`
(an ORM write — correctly left to fall through to `data_access`) or
`document.createElement(...)`. Document this: **state_store matches bare/imported
factory calls only; member-access calls are intentionally not matched.**

**Migration Impact:** Additive; `list_categories()` +1. **Draws from
`framework_hooks` (M35):** `useSelector`/`useDispatch`/`useStore` move from
`framework_hooks` to `state_store` on upgrade — a count shift between two new
categories. Document explicitly in `MIGRATION_NOTES.md` (this is the canonical
"precedence reassignment" example). `snapshot_version` stays `"1.0"`.

**Files to create or modify:**
- **Create:** `crates/sdivi-patterns/src/queries/state_store.rs`.
- **Modify:** `crates/sdivi-patterns/src/queries/mod.rs` (registry order!),
  `crates/sdivi-core/src/categories.rs`.
- **Modify:** `docs/pattern-categories.md`, `MIGRATION_NOTES.md`, `CHANGELOG.md`.

**Acceptance criteria:**
- `useSelector(...)` → `["state_store"]` (NOT `framework_hooks`).
- `createSlice({})` → `["state_store"]`; `useEffect(...)` → `["framework_hooks"]`.
- `category_contract.rs`, WASM count test, clippy/fmt/doc gates green.

**Tests:**
- Unit: store positives; hook/store disambiguation cases; negatives.
- Disjointness corpus: the `use*`-store overlap appears in `KNOWN_OVERLAPS` with
  `state_store` as the documented winner.

**Watch For:**
- **`^`-anchor every generic token.** `create`, `effect`, `action`, `signal`,
  `computed`, `atom` must be anchored at callee start (`^create\(`, not `\bcreate\(`).
  Unanchored, `prisma.user.create(...)` and `document.createElement(...)` would be
  miscategorised. A residual false positive remains for a bare local `create(x)` /
  `effect(x)` that is unrelated to a store — accepted as entropy noise and documented.
- **Order in `CALL_DISPATCH` is load-bearing** — `state_store` must precede
  `framework_hooks`. Add a dedicated test asserting the resolution, not just the regex.

**Seeds Forward:**
- TanStack Query / SWR (`useQuery`, `useMutation`, `useSWR`) blur "state" and
  "data-access". Decide their home in a follow-up (likely `data_access` or a new
  `server_state` category) — out of scope here; note the open question.
