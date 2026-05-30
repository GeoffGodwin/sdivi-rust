
#### Milestone 35: Pattern Category — `framework_hooks`

<!-- milestone-meta
id: "35"
status: "planned"
-->

**Scope:** Add the `framework_hooks` pattern category — component-composition hook
calls in React / Preact / Vue (composables) / Svelte-style runtimes. Detected via
callee-text on `call_expression`: any callee matching `^use[A-Z]` (built-in hooks
`useState`, `useEffect`, `useMemo`, `useCallback`, `useRef`, `useContext`,
`useReducer`, `useLayoutEffect`, plus the entire custom-hook ecosystem). **No
parsing-layer change** — `call_expression` is already collected by the TS/JS
adapters. This is the highest-prevalence construct in modern TS/JS frontend code
and the single best first expansion for the user's priorities.

**Why this milestone exists:** Hook usage is the dominant structural idiom in React
and Vue codebases, and hook-shape variance (which hooks, how many per component,
custom-hook proliferation) is a strong, judgment-free convention-drift signal. It
is the cleanest possible addition: one regex, one registry entry, zero adapter work.

**Deliverables:**
- Create `crates/sdivi-patterns/src/queries/framework_hooks.rs` with
  `NODE_KINDS: &[&str] = &[]` (callee-only) and
  `pub fn matches_callee(text: &str, language: &str) -> bool` backed by a
  `LazyLock<Regex>` for `^use[A-Z]`, active for `typescript`/`javascript` only.
- Register `framework_hooks` in `ALL_CATEGORIES`, in the M34 `CALL_DISPATCH`
  registry at **slot P6** (below `state_store` P5, which M39 inserts above it — see
  Watch For), and in `sdivi-core::categories::CATALOG_ENTRIES` with a description.
- Update `docs/pattern-categories.md`: canonical list, TS/JS node-kind table,
  `framework_hooks::matches_callee` regex table, worked example.

**Detection:**
| Language | Pattern | Examples |
|---|---|---|
| TS / JS | `^use[A-Z]` | `useState(0)`, `useEffect(fn, [])`, `useAuth()`, `useStore()` |
| All others | (none) | — |

**Migration Impact:** Additive category; `list_categories()` count goes 8 → 9.
On first post-upgrade snapshot, TS/JS repos gain a non-zero `framework_hooks`
bucket. Calls previously dropped (unrecognised callee → `[]`) are now classified;
no existing category loses instances (hooks did not match any prior regex).
Document as a count-introduction event in `MIGRATION_NOTES.md`. `snapshot_version`
stays `"1.0"`.

**Files to create or modify:**
- **Create:** `crates/sdivi-patterns/src/queries/framework_hooks.rs`.
- **Modify:** `crates/sdivi-patterns/src/queries/mod.rs` — module + registry entry.
- **Modify:** `crates/sdivi-core/src/categories.rs` — `CATALOG_ENTRIES`.
- **Modify:** `docs/pattern-categories.md`, `MIGRATION_NOTES.md`, `CHANGELOG.md`.

**Acceptance criteria:**
- `classify_hint({call_expression, "useState(0)"}, "typescript") == ["framework_hooks"]`.
- A non-hook call (`username()` lowercase, `getUser()`) does **not** match.
- `list_categories()` length and `category_contract.rs` updated and green.
- WASM `list_categories()` count test updated; clippy/fmt/doc gates pass.

**Tests:**
- Unit: positive (`useMemo`, `useCustomThing`) and negative (`user()`, `используем`,
  `fuse(x)`) callee cases for TS and JS; non-match for python/go/rust/java.
- Disjointness corpus (M34) extended with hook callees.

**Watch For:**
- **Precedence vs `state_store` (M39).** `useSelector`/`useDispatch`/`useStore`
  match both `^use[A-Z]` and the Redux/Zustand regexes. The intent is to count
  those as `state_store` (more specific). M39 must register **above**
  `framework_hooks` in `CALL_DISPATCH`, and the overlap must be listed in
  `KNOWN_OVERLAPS`. Until M39 lands, those resolve to `framework_hooks` — acceptable.
- **`^use[A-Z]` excludes lowercase-second-char** by design (`user`, `useful` won't
  match; `useX` will). Anchor at start to avoid mid-identifier matches.
- **Naming.** `framework_hooks`, not `react_hooks` — reserved-forever; keep it
  framework-neutral so Vue composables / Svelte fit the same bucket.

**Seeds Forward:**
- Class-component lifecycle (`componentDidMount`, etc.) could fold in later via a
  broadened regex if legacy React coverage is requested. Defer until asked.
