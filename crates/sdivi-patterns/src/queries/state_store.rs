//! Callee-text classification for external state-management library declarations.
//!
//! Detects library-level state-store idioms in TypeScript and JavaScript, distinct
//! from the closure/arrow-function bucket (`state_management`). Covered families:
//!
//! - **Redux / RTK:** `createSlice`, `configureStore`, `createStore`, `combineReducers`,
//!   `createAsyncThunk`, `createReducer`, `createAction`.
//! - **React-Redux hooks:** `useSelector`, `useDispatch`, `useStore` — these match
//!   `^use[A-Z]` (the `framework_hooks` regex) too; state_store wins via CALL_DISPATCH
//!   precedence (P5 < P6). See `KNOWN_OVERLAPS` in `dispatch_disjointness.rs`.
//! - **Zustand:** bare `create(` — anchored at callee start so `prisma.user.create(...)`
//!   (ORM write) and `document.createElement(...)` are intentionally not matched.
//! - **Jotai / Recoil:** `atom`, `selector`, `atomFamily`, `selectorFamily`.
//! - **MobX:** `observable`, `action`, `computed`, `makeObservable`,
//!   `makeAutoObservable`, `runInAction`.
//! - **Signals (Preact/Angular):** `signal`, `computed`, `effect`, `batch`.
//! - **Solid:** `createSignal`, `createEffect`, `createMemo`, `createStore`,
//!   `createResource`.
//!
//! All patterns are `^`-anchored at callee start: imported factory calls are invoked
//! bare (`create(...)`, `atom(0)`, `signal(0)`), so the anchor captures them while
//! excluding member-access calls (`prisma.user.create(...)` — ORM write, left to
//! `data_access`; `document.createElement(...)` — DOM API, left to unclassified).
//!
//! **Accepted noise:** a bare local function named `create(x)`, `effect(x)`, or
//! `atom(x)` unrelated to a store will be miscategorised. Treated as entropy noise
//! — the signal is the population of store factories at codebase scale, not each call.
//!
//! **Open question (TanStack Query / SWR):** `useQuery`, `useMutation`, `useSWR` blur
//! "state" and "data-fetching". Their home — `data_access`, `state_store`, or a future
//! `server_state` category — is deferred to a follow-up milestone. Currently they fall
//! through to `framework_hooks` (via the `^use[A-Z]` regex) until that decision lands.
//!
//! Detection is callee-text only — no tree-sitter node-kind matching. `call_expression`
//! nodes are collected by the TS/JS adapters; this module provides callee-text
//! discrimination in `CALL_DISPATCH` at slot P5.

use std::sync::LazyLock;

use regex::Regex;

/// Tree-sitter node kinds for state-store patterns.
///
/// Empty — this category is detected entirely via callee-text inspection in
/// [`matches_callee`]. The `call_expression` node kind is already collected
/// by the TypeScript/JavaScript adapters; classification happens in
/// `classify_hint`'s `CALL_DISPATCH` loop at slot P5.
pub const NODE_KINDS: &[&str] = &[];

// TypeScript / JavaScript — all patterns are `^`-anchored at callee start.
//
// Group 1 — RTK / Redux factories and Solid createX factories:
//   ^(createSlice|configureStore|createStore|combineReducers|
//     createAsyncThunk|createReducer|createAction|
//     createSignal|createEffect|createMemo|createResource)\(
//
// Group 2 — React-Redux hooks (these also match framework_hooks' ^use[A-Z];
//           state_store wins via P5 < P6 precedence in CALL_DISPATCH):
//   ^use(Selector|Dispatch|Store)\b
//
// Group 3 — Zustand bare create:
//   ^create\(
//
// Group 4 — Jotai / Recoil:
//   ^(atom|selector|atomFamily|selectorFamily)\(
//
// Group 5 — MobX + Signals (computed/effect/signal appear in both families):
//   ^(observable|action|computed|makeObservable|makeAutoObservable|runInAction|signal|effect|batch)\(
static TS_JS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(createSlice|configureStore|createStore|combineReducers|createAsyncThunk|createReducer|createAction|createSignal|createEffect|createMemo|createResource)\(|^use(Selector|Dispatch|Store)\b|^create\(|^(atom|selector|atomFamily|selectorFamily)\(|^(observable|action|computed|makeObservable|makeAutoObservable|runInAction|signal|effect|batch)\(",
    )
    .expect("state_store TS/JS regex is valid")
});

/// Return `true` when `text` looks like a state-store callee for `language`.
///
/// Matches TypeScript and JavaScript only — state-store library conventions
/// (`createSlice`, `atom`, `signal`, etc.) are specific to the JS/TS ecosystem.
/// All other languages return `false` in v0.
///
/// # Examples
///
/// ```rust
/// use sdivi_patterns::queries::state_store::matches_callee;
///
/// // Redux / RTK
/// assert!(matches_callee("createSlice({})", "typescript"));
/// assert!(matches_callee("configureStore({})", "typescript"));
/// // React-Redux hooks (P5 beats framework_hooks P6)
/// assert!(matches_callee("useSelector(s => s.x)", "typescript"));
/// assert!(matches_callee("useDispatch()", "javascript"));
/// // Zustand — bare create, NOT member call
/// assert!(matches_callee("create((set) => ({}))", "typescript"));
/// // Jotai / Recoil
/// assert!(matches_callee("atom(0)", "typescript"));
/// // MobX
/// assert!(matches_callee("makeAutoObservable(this)", "typescript"));
/// // Signals
/// assert!(matches_callee("signal(0)", "typescript"));
/// // NOT matched: member-access calls
/// assert!(!matches_callee("prisma.user.create(data)", "typescript"));
/// assert!(!matches_callee("document.createElement('div')", "typescript"));
/// // NOT matched: unrelated hooks — falls through to framework_hooks
/// assert!(!matches_callee("useEffect(fn, [])", "typescript"));
/// ```
pub fn matches_callee(text: &str, language: &str) -> bool {
    match language {
        "typescript" | "javascript" => TS_JS_RE.is_match(text),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redux_rtk_factories_match() {
        for callee in [
            "createSlice({})",
            "configureStore({})",
            "createStore(rootReducer)",
            "combineReducers({})",
            "createAsyncThunk('fetch', async () => {})",
            "createReducer(init, builder => {})",
            "createAction('increment')",
        ] {
            assert!(
                matches_callee(callee, "typescript"),
                "{callee:?} should match for typescript"
            );
        }
    }

    #[test]
    fn react_redux_hooks_match() {
        assert!(matches_callee("useSelector(s => s.user)", "typescript"));
        assert!(matches_callee("useDispatch()", "typescript"));
        assert!(matches_callee("useStore()", "javascript"));
    }

    #[test]
    fn zustand_create_matches() {
        assert!(matches_callee("create((set) => ({}))", "typescript"));
        assert!(matches_callee("create(() => ({count: 0}))", "javascript"));
    }

    #[test]
    fn jotai_recoil_match() {
        assert!(matches_callee("atom(0)", "typescript"));
        assert!(matches_callee(
            "selector({key: 'x', get: () => 1})",
            "typescript"
        ));
        assert!(matches_callee("atomFamily((id) => atom(id))", "typescript"));
        assert!(matches_callee(
            "selectorFamily({key: 'x', get: () => () => 1})",
            "typescript"
        ));
    }

    #[test]
    fn mobx_primitives_match() {
        assert!(matches_callee("observable({})", "typescript"));
        assert!(matches_callee("action(() => {})", "typescript"));
        assert!(matches_callee("computed(() => x + 1)", "typescript"));
        assert!(matches_callee("makeObservable(this, {})", "typescript"));
        assert!(matches_callee("makeAutoObservable(this)", "typescript"));
        assert!(matches_callee("runInAction(() => {})", "typescript"));
    }

    #[test]
    fn signals_match() {
        assert!(matches_callee("signal(0)", "typescript"));
        assert!(matches_callee(
            "computed(() => count.value * 2)",
            "typescript"
        ));
        assert!(matches_callee("effect(() => {})", "typescript"));
        assert!(matches_callee("batch(() => {})", "typescript"));
    }

    #[test]
    fn solid_createx_match() {
        assert!(matches_callee("createSignal(0)", "typescript"));
        assert!(matches_callee("createEffect(() => {})", "typescript"));
        assert!(matches_callee("createMemo(() => x)", "typescript"));
        assert!(matches_callee("createStore({})", "typescript"));
        assert!(matches_callee("createResource(fetcher)", "typescript"));
    }

    #[test]
    fn member_access_calls_do_not_match() {
        // Zustand anchor: member-access paths are excluded
        assert!(!matches_callee("prisma.user.create(data)", "typescript"));
        assert!(!matches_callee("db.create(record)", "typescript"));
        // DOM API — excluded
        assert!(!matches_callee(
            "document.createElement('div')",
            "typescript"
        ));
        // ORM — excluded
        assert!(!matches_callee("repo.createMany(items)", "typescript"));
    }

    #[test]
    fn unrelated_use_hooks_do_not_match() {
        // useEffect / useState etc. are framework_hooks, not state_store
        assert!(!matches_callee("useEffect(fn, [])", "typescript"));
        assert!(!matches_callee("useState(0)", "typescript"));
        assert!(!matches_callee("useMemo(() => v, [])", "typescript"));
        assert!(!matches_callee("useCustomHook()", "typescript"));
    }

    #[test]
    fn other_languages_return_false() {
        for lang in ["python", "rust", "go", "java"] {
            assert!(
                !matches_callee("atom(0)", lang),
                "atom should not match for {lang}"
            );
            assert!(
                !matches_callee("createSlice({})", lang),
                "createSlice should not match for {lang}"
            );
        }
    }

    #[test]
    fn node_kinds_is_empty() {
        // NODE_KINDS is intentionally empty: this category is callee-only (classified
        // via classify_hint). The assertion guards that contract against regressions.
        #[allow(clippy::const_is_empty)]
        let empty = NODE_KINDS.is_empty();
        assert!(empty);
    }
}
