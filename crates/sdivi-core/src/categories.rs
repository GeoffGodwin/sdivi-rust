//! Canonical pattern-category contract for sdivi-rust `snapshot_version "1.0"`.
//!
//! Embedders that supply their own tree-sitter extractors (e.g. the Meridian consumer app)
//! must use the category names returned by [`list_categories`] so that
//! [`crate::compute_pattern_metrics`] and [`crate::compute_delta`] produce meaningful
//! per-category divergence values.

use sdivi_snapshot::snapshot::SNAPSHOT_VERSION;
use serde::{Deserialize, Serialize};

/// Single source of truth: (name, description) for every canonical category.
///
/// Every entry here is a permanent contract entry for `snapshot_version "1.0"`.
/// Once a name is in this list it cannot be removed — only deprecated.
/// [`CATEGORIES`] and [`list_categories`] are derived from this array and cannot silently diverge.
const CATALOG_ENTRIES: &[(&str, &str)] = &[
    (
        "async_patterns",
        "Code constructs that implement or leverage asynchronous execution — \
        e.g., `.await` expressions on `Future` values and `async fn` definitions.",
    ),
    (
        "class_hierarchy",
        "Code constructs that establish inheritance, interface implementation, or trait \
        conformance relationships — e.g. classes with `extends`/`implements` clauses, \
        Python classes with base classes, and Rust `impl Trait for Type` blocks. All \
        declaration kinds are classified here regardless of whether they carry a \
        heritage clause; heritage-aware narrowing is the embedder's responsibility.",
    ),
    (
        "collection_pipelines",
        "Functional collection-transform method calls — `.map`, `.filter`, `.reduce`, \
        `.flatMap`, `.forEach`, `.find`, `.findIndex`, `.some`, `.every`, `.flat`. \
        Detected via member-call callee-text on `call_expression` at CALL_DISPATCH \
        slot P10 (broadest member-call category — more specific categories resolve \
        first). Callee-text cannot distinguish the receiver type: `rxObservable.map(fn)`, \
        `new Map().forEach(cb)`, and `array.map(f)` all match — treated as acceptable \
        noise for an entropy measure. Bare calls without a dot prefix (`map(f)`) are \
        intentionally not matched. TypeScript and JavaScript primary targets; the same \
        regex applies to Go and Java where these method names appear. Added M40.",
    ),
    (
        "comprehensions",
        "Python comprehension and generator forms (`list_comprehension`, \
        `set_comprehension`, `dictionary_comprehension`, `generator_expression`). \
        Node-kind only; Python-only in v0; nested comprehensions each emit their own node. Added M46.",
    ),
    (
        "concurrency",
        "Concurrent-execution primitives distinct from single-future async patterns — \
        Go goroutine launches (`go_statement`) and channel multiplexing (`select_statement`), \
        plus multi-future coordination calls: `Promise.all/allSettled/race/any` \
        (TypeScript/JavaScript) and `asyncio.gather/create_task/wait/as_completed/run` \
        (Python). Detected via node kind (Go) and callee-text at CALL_DISPATCH slot P11 \
        (TS/JS and Python). Added M44.",
    ),
    (
        "data_access",
        "Code constructs that perform I/O against data stores or external resources — \
        e.g., database queries (`query`, `cursor.*`), HTTP fetches (`fetch`), \
        file reads (`open`, `read`), and ORM method calls. All `call_expression` / \
        `call` nodes are classified here; callee-name narrowing is the embedder's \
        responsibility.",
    ),
    (
        "decorators",
        "TypeScript and JavaScript decorator syntax (`@Injectable()`, `@Component({...})`, \
        `@Entity()`, `@Get('/')`, `@IsString()`, etc.). Every `decorator` node counts — \
        broad collection in the spirit of `class_hierarchy`. Decorator-shape entropy is \
        the signal; no callee allowlist is applied. Added M36.1.",
    ),
    (
        "error_handling",
        "Code constructs that propagate, transform, or handle error conditions — \
        e.g., the `?` operator (`try_expression`) and `match` arms that dispatch \
        on `Result` or `Option` variants.",
    ),
    (
        "framework_hooks",
        "Component-composition hook calls in React, Preact, Vue (composables), and \
        Svelte-style runtimes — any `call_expression` callee matching `^use[A-Z]` in \
        TypeScript or JavaScript. Built-in hooks (`useState`, `useEffect`, `useMemo`, \
        `useCallback`, `useRef`, `useContext`, `useReducer`, `useLayoutEffect`) and the \
        custom-hook ecosystem (`useAuth`, `useTheme`, etc.). Note: `useSelector`, \
        `useDispatch`, `useStore` route to `state_store` (P5 > P6). Other languages: none.",
    ),
    (
        "http_routing",
        "Server-side HTTP route/endpoint registration calls — Express/Koa/Fastify/Hono \
        (`app.get`, `router.post`, `fastify.route`, `server.use`, `srv.all`), \
        Go net/http + Gin/Echo/Gorilla \
        (`http.HandleFunc`, `r.GET`, `mux.Handle`, `e.POST`, `engine.Group`), and \
        Flask/FastAPI imperative registration (`app.add_url_rule`). Detection is \
        receiver-allowlist anchored: only calls whose receiver is a known server/router \
        handle are matched, so client HTTP calls (`axios.get`, `fetch`) stay in \
        `data_access`. NestJS and FastAPI decorator routes (`@Get('/')`, \
        `@app.get(...)`) are `decorator`/`decorated_definition` nodes classified under \
        `decorators` (M36.1/M36.2). Registered at CALL_DISPATCH slot P7, above \
        `logging` (P8) and `data_access` (P9). Added M41.",
    ),
    (
        "logging",
        "Code constructs that produce diagnostic or observability output — \
        e.g., `console.*` calls, structured logger invocations (`logger.info`), \
        `print` statements, and logging macros (`tracing::info!`, `log::debug!`). \
        Natively classified since M33 via `classify_hint` callee-text inspection: \
        `category_for_node_kind` does not return `Some(\"logging\")` (the relevant \
        node kinds overlap with `data_access` and `resource_management`), but \
        `classify_hint` routes matching callees to this category. Foreign extractors \
        may also emit `PatternInstanceInput { category: \"logging\", … }` directly; \
        those instances merge with natively classified ones.",
    ),
    (
        "null_safety",
        "Code constructs that guard against null or undefined values — optional \
        chaining (`a?.b`, `arr?.[0]`) via `optional_chain` and TypeScript non-null \
        assertions (`el!`) via `non_null_expression`. Optional calls (`fn?.()`) \
        emit `call_expression` in the grammar and are not counted here. \
        TypeScript and JavaScript only; nullish coalescing (`??`) is deferred. \
        Added M37.",
    ),
    (
        "resource_management",
        "Code constructs that allocate, release, or manage system or heap resources — \
        e.g., macro invocations such as `drop!`, `vec!`, or standard I/O macros.",
    ),
    (
        "schema_validation",
        "Runtime schema and validation declarations — Zod (`z.object`, `z.string`), \
        Yup (`yup.object().shape(...)`), Valibot (`v.object`), Superstruct (`s.object`), \
        and the Zod-specific `.safeParse(` call in TypeScript and JavaScript. Python: \
        Pydantic field-constraint calls (`Field(...)`, `constr(...)`, `conint(...)`). \
        Detected via callee-text on `call_expression`/`call` at CALL_DISPATCH slot P4. \
        `class Foo(BaseModel)` is a `class_definition` counted under `class_hierarchy`. \
        class-validator decorators (`@IsString()`) belong to `decorators` (M36.1/M36.2). \
        TypeScript, JavaScript, and Python only; other languages produce no instances in v0. \
        Added M38.",
    ),
    (
        "serialization",
        "Serialization boundary calls — `JSON.parse`, `JSON.stringify`, `structuredClone` \
        (TypeScript/JavaScript); `json.loads`, `json.dumps`, `json.load`, `json.dump`, \
        `pickle.loads`, `pickle.dumps` (Python); `json.Marshal`, `json.Unmarshal`, \
        `json.MarshalIndent`, `json.NewEncoder`, `json.NewDecoder` (Go). \
        Receiver-anchored: only `JSON.`, `json.`, or `pickle.` callee prefixes are matched; \
        bare `.parse(` is intentionally excluded (collides with schema validators). \
        Detected via callee-text at CALL_DISPATCH slot P3 (above `schema_validation` P4). Added M43.",
    ),
    (
        "state_management",
        "Code constructs that capture, transform, or share mutable or shared state — \
        e.g., closures that close over mutable bindings or shared references.",
    ),
    (
        "state_store",
        "External state-management library declarations — Redux / RTK \
        (`createSlice`, `configureStore`, `createStore`, `combineReducers`, \
        `createAsyncThunk`), React-Redux hooks (`useSelector`, `useDispatch`, \
        `useStore`), Zustand (`create(...)`), Jotai / Recoil (`atom`, `selector`, \
        `atomFamily`, `selectorFamily`), MobX (`observable`, `action`, `computed`, \
        `makeObservable`, `makeAutoObservable`, `runInAction`), Signals — Preact/Angular \
        (`signal`, `computed`, `effect`, `batch`), and Solid (`createSignal`, \
        `createEffect`, `createMemo`, `createStore`, `createResource`). \
        Detected via callee-text on `call_expression` at CALL_DISPATCH slot P5 (above \
        `framework_hooks` P6). All patterns are `^`-anchored at callee start — \
        member-access calls (`prisma.user.create(...)`, `document.createElement(...)`) \
        are intentionally not matched. TypeScript and JavaScript only in v0. \
        `useSelector`, `useDispatch`, and `useStore` match both this category and \
        `framework_hooks`; `state_store` wins via precedence (P5 < P6). \
        Added M39.",
    ),
    (
        "testing",
        "Test-suite structure and assertion calls — BDD suite globals (`describe`, `it`, \
        `context`), flat `test` globals, lifecycle hooks (`beforeEach`, `afterEach`, \
        `beforeAll`, `afterAll`), `expect(…)` assertion roots, focused/excluded variants \
        (`xit`, `xdescribe`, `fit`, `fdescribe`), and framework-namespaced helpers \
        (`jest.fn`, `jest.mock`, `jest.spyOn`, `vi.fn`, `vi.mock`, `vi.spyOn`, etc.) in \
        TypeScript and JavaScript (Jest/Vitest/Mocha/Jasmine). Go `testing.T` method calls \
        (`t.Run`, `t.Fatal`, `t.Error`, `t.Errorf`, and the full T method set). Python \
        `unittest.TestCase` assertion methods (`self.assertEqual`, `self.assertTrue`, and \
        the full `self.assert[A-Z]…` family). Detected via callee-text at CALL_DISPATCH \
        slot P2 (above `schema_validation` P4). **`scope_exclude` interaction:** the \
        `testing` bucket is non-empty only when test files are in the pattern scope; \
        repos that exclude test paths via `patterns.scope_exclude` will see a zero \
        count. Added M42.",
    ),
    (
        "type_assertions",
        "Code constructs that assert or coerce between types at compile or runtime — \
        e.g., `as` casts (`as_expression`) and language-specific type-cast expressions.",
    ),
];

/// Canonical category names in stable alphabetical order.
///
/// Every name here is a permanent contract entry for `snapshot_version "1.0"`.
/// Once a name is in this list it cannot be removed — only deprecated.
///
/// # Examples
///
/// ```rust
/// use sdivi_core::CATEGORIES;
///
/// assert!(CATEGORIES.contains(&"comprehensions"));
/// assert_eq!(CATEGORIES.len(), 19);
/// ```
pub const CATEGORIES: &[&str] = &[
    CATALOG_ENTRIES[0].0,
    CATALOG_ENTRIES[1].0,
    CATALOG_ENTRIES[2].0,
    CATALOG_ENTRIES[3].0,
    CATALOG_ENTRIES[4].0,
    CATALOG_ENTRIES[5].0,
    CATALOG_ENTRIES[6].0,
    CATALOG_ENTRIES[7].0,
    CATALOG_ENTRIES[8].0,
    CATALOG_ENTRIES[9].0,
    CATALOG_ENTRIES[10].0,
    CATALOG_ENTRIES[11].0,
    CATALOG_ENTRIES[12].0,
    CATALOG_ENTRIES[13].0,
    CATALOG_ENTRIES[14].0,
    CATALOG_ENTRIES[15].0,
    CATALOG_ENTRIES[16].0,
    CATALOG_ENTRIES[17].0,
    CATALOG_ENTRIES[18].0,
];

// Compile-time length guard — add a new index to CATEGORIES when CATALOG_ENTRIES grows.
const _: () = assert!(CATEGORIES.len() == CATALOG_ENTRIES.len(), "CATEGORIES and CATALOG_ENTRIES out of sync");

/// Metadata for a single canonical pattern category.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CategoryInfo {
    /// Canonical category name — stable across all `snapshot_version "1.0"` output.
    pub name: String,
    /// Human-readable description of the code constructs this category covers.
    pub description: String,
}

/// Runtime representation of the canonical pattern-category contract.
///
/// Returned by [`list_categories`]. Embedders should call this function
/// instead of hard-coding category names so they stay aligned with the contract.
///
/// # Examples
///
/// ```rust
/// use sdivi_core::list_categories;
///
/// let catalog = list_categories();
/// assert_eq!(catalog.schema_version, "1.0");
/// assert!(!catalog.categories.is_empty());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CategoryCatalog {
    /// The `snapshot_version` string this contract applies to.
    pub schema_version: String,
    /// All canonical categories in alphabetical order.
    pub categories: Vec<CategoryInfo>,
}

/// Return the canonical pattern-category contract for `snapshot_version "1.0"`.
///
/// The returned [`CategoryCatalog`] is the authoritative source of truth for:
///
/// - Which category names are valid in [`crate::input::PatternInstanceInput::category`].
/// - Which category names appear as keys in per-category divergence maps.
/// - Which category names are accepted by `[thresholds.overrides.<cat>]` in `config.toml`.
///
/// Embedders MUST use these names verbatim (case-sensitive). Referentially transparent.
///
/// # Examples
///
/// ```rust
/// use sdivi_core::list_categories;
///
/// let a = list_categories();
/// let b = list_categories();
/// assert_eq!(a, b, "list_categories must be referentially transparent");
/// assert_eq!(a.schema_version, "1.0");
///
/// let names: Vec<&str> = a.categories.iter().map(|c| c.name.as_str()).collect();
/// assert!(names.contains(&"comprehensions"));
/// assert!(names.contains(&"error_handling"));
/// ```
pub fn list_categories() -> CategoryCatalog {
    CategoryCatalog {
        schema_version: SNAPSHOT_VERSION.to_string(),
        categories: CATALOG_ENTRIES
            .iter()
            .map(|(name, desc)| CategoryInfo {
                name: name.to_string(),
                description: desc.to_string(),
            })
            .collect(),
    }
}
