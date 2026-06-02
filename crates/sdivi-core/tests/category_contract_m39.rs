//! M39 acceptance criterion tests — `state_store` pattern category.
//!
//! Verifies that:
//! - `useSelector(...)` → `["state_store"]` (NOT `framework_hooks`)
//! - `createSlice({})` → `["state_store"]`
//! - `useEffect(...)` → `["framework_hooks"]` (disambiguation)
//! - Member-access calls do not match state_store

use sdivi_patterns::queries::classify_hint;
use sdivi_patterns::PatternHintInput;

fn call_hint(text: &str) -> PatternHintInput {
    PatternHintInput {
        node_kind: "call_expression".to_string(),
        text: text.to_string(),
    }
}

// ── Redux / RTK ───────────────────────────────────────────────────────────────

#[test]
fn create_slice_typescript_is_state_store() {
    assert_eq!(
        classify_hint(&call_hint("createSlice({})"), "typescript"),
        vec!["state_store"],
        "createSlice must classify as state_store"
    );
}

#[test]
fn configure_store_javascript_is_state_store() {
    assert_eq!(
        classify_hint(&call_hint("configureStore({})"), "javascript"),
        vec!["state_store"]
    );
}

#[test]
fn create_async_thunk_is_state_store() {
    assert_eq!(
        classify_hint(
            &call_hint("createAsyncThunk('users/fetch', async (id) => {})"),
            "typescript"
        ),
        vec!["state_store"]
    );
}

// ── React-Redux hooks — P5 beats P6 ──────────────────────────────────────────

#[test]
fn use_selector_is_state_store_not_framework_hooks() {
    let result = classify_hint(&call_hint("useSelector(s => s.user)"), "typescript");
    assert_eq!(
        result,
        vec!["state_store"],
        "useSelector must resolve to state_store (P5 beats framework_hooks P6)"
    );
}

#[test]
fn use_dispatch_is_state_store() {
    assert_eq!(
        classify_hint(&call_hint("useDispatch()"), "typescript"),
        vec!["state_store"]
    );
}

#[test]
fn use_store_is_state_store() {
    assert_eq!(
        classify_hint(&call_hint("useStore()"), "javascript"),
        vec!["state_store"]
    );
}

// ── Disambiguation: unrelated hooks still resolve to framework_hooks ──────────

#[test]
fn use_effect_is_framework_hooks_not_state_store() {
    let result = classify_hint(&call_hint("useEffect(fn, [])"), "typescript");
    assert_eq!(
        result,
        vec!["framework_hooks"],
        "useEffect must remain framework_hooks — state_store only captures use(Selector|Dispatch|Store)"
    );
}

#[test]
fn use_state_is_framework_hooks_not_state_store() {
    let result = classify_hint(&call_hint("useState(0)"), "typescript");
    assert_eq!(result, vec!["framework_hooks"]);
}

#[test]
fn use_auth_is_framework_hooks_not_state_store() {
    let result = classify_hint(&call_hint("useAuth()"), "typescript");
    assert_eq!(result, vec!["framework_hooks"]);
}

// ── Zustand / Jotai / MobX / Signals / Solid ─────────────────────────────────

#[test]
fn zustand_create_is_state_store() {
    assert_eq!(
        classify_hint(&call_hint("create((set) => ({}))"), "typescript"),
        vec!["state_store"]
    );
}

#[test]
fn atom_is_state_store() {
    assert_eq!(
        classify_hint(&call_hint("atom(0)"), "typescript"),
        vec!["state_store"]
    );
}

#[test]
fn make_auto_observable_is_state_store() {
    assert_eq!(
        classify_hint(&call_hint("makeAutoObservable(this)"), "typescript"),
        vec!["state_store"]
    );
}

#[test]
fn signal_is_state_store() {
    assert_eq!(
        classify_hint(&call_hint("signal(0)"), "typescript"),
        vec!["state_store"]
    );
}

#[test]
fn create_signal_solid_is_state_store() {
    assert_eq!(
        classify_hint(&call_hint("createSignal(0)"), "typescript"),
        vec!["state_store"]
    );
}

// ── Member-access calls must not match ───────────────────────────────────────

#[test]
fn prisma_create_is_not_state_store() {
    let result = classify_hint(&call_hint("prisma.user.create(data)"), "typescript");
    assert!(
        !result.contains(&"state_store"),
        "prisma.user.create must not match state_store — member-access ORM calls excluded"
    );
}

#[test]
fn document_create_element_is_not_state_store() {
    let result = classify_hint(&call_hint("document.createElement('div')"), "typescript");
    assert!(
        !result.contains(&"state_store"),
        "document.createElement must not match state_store"
    );
}

// ── Wrong language — no state_store ──────────────────────────────────────────

#[test]
fn state_store_does_not_fire_for_python() {
    let result = classify_hint(&call_hint("atom(0)"), "python");
    assert!(
        !result.contains(&"state_store"),
        "state_store must not fire for python"
    );
}

#[test]
fn list_categories_includes_state_store() {
    let catalog = sdivi_core::list_categories();
    let names: Vec<&str> = catalog.categories.iter().map(|c| c.name.as_str()).collect();
    assert!(
        names.contains(&"state_store"),
        "list_categories must include 'state_store'"
    );
}

// ── Native name-level assertions for categories absent from dedicated tests ───
// These parallel the wasm_smoke.rs name assertions; running under `cargo test`
// means CI catches regressions even without a wasm-pack test run.

#[test]
fn list_categories_includes_resource_management() {
    let catalog = sdivi_core::list_categories();
    let names: Vec<&str> = catalog.categories.iter().map(|c| c.name.as_str()).collect();
    assert!(
        names.contains(&"resource_management"),
        "list_categories must include 'resource_management'"
    );
}

#[test]
fn list_categories_includes_state_management() {
    let catalog = sdivi_core::list_categories();
    let names: Vec<&str> = catalog.categories.iter().map(|c| c.name.as_str()).collect();
    assert!(
        names.contains(&"state_management"),
        "list_categories must include 'state_management'"
    );
}

#[test]
fn list_categories_includes_type_assertions() {
    let catalog = sdivi_core::list_categories();
    let names: Vec<&str> = catalog.categories.iter().map(|c| c.name.as_str()).collect();
    assert!(
        names.contains(&"type_assertions"),
        "list_categories must include 'type_assertions'"
    );
}
