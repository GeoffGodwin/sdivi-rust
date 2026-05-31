# Pattern Category Contract

**Applies to:** `snapshot_version "1.0"`

## Versioning

Pattern categories are bound to `snapshot_version "1.0"`. The rules are:

- **Reserved forever once introduced.** A category name that appears in `list_categories()` cannot be removed within a snapshot version — only marked deprecated in its description. Embedders may have stored snapshots referencing it, and `compute_delta` must keep working.
- **Additive within a snapshot version.** New categories may be added; existing names and their meanings may not change.
- **Removed-category procedure.** If a category must truly be retired (not just deprecated), that requires bumping `snapshot_version` to a new value per Rule 16 of CLAUDE.md. Document the bump procedure in `MIGRATION_NOTES.md`.
- **Per-language node-kind tables are an implementation detail.** The `snapshot_version "1.0"` contract covers the *category set*. Node-kind strings can grow as new language adapters are added without a version bump.

The authoritative runtime source of truth is `sdivi_core::list_categories()`. The table below is generated from the same constant — the [category contract test](../crates/sdivi-core/tests/category_contract.rs) asserts they match.

## Canonical category list

| Category | Description |
|---|---|
| async_patterns | Code constructs that implement or leverage asynchronous execution — e.g., `.await` expressions on `Future` values and `async fn` definitions. In TypeScript/JavaScript, Promise-chain calls (`.then()`, `.catch()`, `.finally()`) are also classified here via callee-text inspection. |
| class_hierarchy | Code constructs that establish inheritance, interface implementation, or trait conformance relationships — e.g. classes with `extends`/`implements` clauses, Python classes with base classes, and Rust `impl Trait for Type` blocks. All declaration kinds are classified here regardless of whether they carry a heritage clause; heritage-aware narrowing is the embedder's responsibility. |
| collection_pipelines | Functional collection-transform method calls — `.map`, `.filter`, `.reduce`, `.flatMap`, `.forEach`, `.find`, `.findIndex`, `.some`, `.every`, `.flat`. Detected via member-call callee-text at CALL_DISPATCH slot P10 (broadest member-call category — more specific categories resolve first). Callee-text cannot distinguish the receiver type: `rxObservable.map(fn)`, `new Map().forEach(cb)`, and `array.map(f)` all match — treated as acceptable noise for an entropy measure. Bare calls without a dot prefix are not matched. TypeScript and JavaScript primary targets; the regex also matches Go/Java where these method names appear. Added M40. |
| data_access | Code constructs that perform I/O against data stores or external resources — e.g., database queries (`db.query`, `cursor.*`), HTTP fetches (`fetch`, `axios`), file reads (`open`, `requests.*`). As of M33, only `call_expression`/`call` nodes whose callee text matches the per-language data-access regex are classified here; unrecognised callees are dropped. |
| decorators | Decorator usage across languages. TypeScript/JavaScript: any `decorator` node (`@Injectable()`, `@Component({...})`, `@Entity()`, `@Get('/')`, `@IsString()`, etc.) — one instance per decorator line. Python: any `decorated_definition` wrapper node (`@dataclass`, `@property`, `@app.get(...)`, `@pytest.fixture`, `@cached_property`, etc.) — one instance per decorated function or class (wrapper-granularity). Added M36.1 (TS/JS); M36.2 (Python). |
| error_handling | Code constructs that propagate, transform, or handle error conditions — e.g., the `?` operator (`try_expression`) and `match` arms that dispatch on `Result` or `Option` variants. |
| framework_hooks | Component-composition hook calls in React, Preact, Vue (composables), and Svelte-style runtimes — any `call_expression` callee matching `^use[A-Z]` in TypeScript or JavaScript. Covers built-in hooks (`useState`, `useEffect`, `useMemo`, `useCallback`, `useRef`, `useContext`, `useReducer`, `useLayoutEffect`) and the full custom-hook ecosystem. Added M35. |
| http_routing | Server-side HTTP route/endpoint registration calls — Express/Koa/Fastify (`app.get`, `router.post`, `fastify.route`, `server.use`), Hono, Go net/http + Gin/Echo/Gorilla (`http.HandleFunc`, `r.GET`, `mux.Handle`, `e.POST`), and Flask/FastAPI imperative registration (`app.add_url_rule`). Detection is receiver-allowlist anchored: only calls whose receiver is a known server/router handle are matched, so client HTTP calls (`axios.get`, `fetch`) stay in `data_access`. NestJS and FastAPI decorator routes (`@Get('/')`, `@app.get(...)`) are `decorator`/`decorated_definition` nodes classified under `decorators` (M36.1/M36.2). Registered at CALL_DISPATCH slot P7. Added M41. |
| logging | Code constructs that produce diagnostic or observability output — e.g., `console.*` calls, structured logger invocations (`logger.info`), `print` statements, and logging macros (`tracing::info!`, `log::debug!`). **Natively classified since M33** via callee-text inspection in `classify_hint` — see Callee-text classification section. |
| null_safety | Code constructs that guard against null or undefined values — optional chaining (`a?.b`, `arr?.[0]`, `fn?.()`) and TypeScript non-null assertions (`el!`). TypeScript and JavaScript only; other languages produce no instances in v0. Nullish coalescing (`??`) is deferred. Added M37. |
| resource_management | Code constructs that allocate, release, or manage system or heap resources — e.g., Rust macro invocations such as `drop!`, `vec!`, `assert!`. As of M33, Rust logging macros (`tracing::*!`, `log::*!`, `println!`/`eprintln!`/`print!`/`eprint!`/`dbg!`) are excluded and classified as `logging` instead. |
| schema_validation | Runtime schema and validation declarations — Zod (`z.object`, `z.string`, `z.enum`), Yup (`yup.object().shape(...)`), Valibot (`v.object`, `v.pipe`), Superstruct (`s.object`), and the Zod-specific `.safeParse(` validated-parse call in TypeScript and JavaScript. Python: Pydantic field-constraint calls (`Field(...)`, `constr(...)`, `conint(...)`). Detected via callee-text at CALL_DISPATCH slot P4. Note: `class Foo(BaseModel)` is a `class_definition` counted under `class_hierarchy`; class-validator decorators (`@IsString()`) belong to `decorators` (M36.1/M36.2). TypeScript, JavaScript, and Python only in v0. Added M38. |
| state_management | Code constructs that capture, transform, or share mutable or shared state — e.g., closures that close over mutable bindings or shared references. |
| state_store | External state-management library declarations — Redux / RTK (`createSlice`, `configureStore`, `createStore`, etc.), React-Redux hooks (`useSelector`, `useDispatch`, `useStore`), Zustand (`create(...)`), Jotai / Recoil (`atom`, `selector`), MobX (`observable`, `makeAutoObservable`, etc.), Signals (`signal`, `computed`, `effect`), and Solid (`createSignal`, `createStore`, etc.). Detected via callee-text at CALL_DISPATCH slot P5 (above `framework_hooks` P6). All patterns are `^`-anchored — member-access calls (`prisma.user.create(...)`) are not matched. `useSelector`/`useDispatch`/`useStore` move from `framework_hooks` to `state_store` on upgrade (see `MIGRATION_NOTES.md`). TypeScript and JavaScript only in v0. Added M39. |
| type_assertions | Code constructs that assert or coerce between types at compile or runtime — e.g., `as` casts (`as_expression`) and language-specific type-cast expressions. |

## Per-language node-kind mappings

Each cell lists the tree-sitter node-kind strings that map to that category in a given language. The current classification is language-unaware (the `language` parameter in `category_for_node_kind` is reserved for future per-language overrides). All supported languages share the same node-kind table.

### Rust

| Category | Node kinds | Structural constraint |
|---|---|---|
| async_patterns | `await_expression` | None |
| class_hierarchy | `impl_item` | All `impl` blocks, including inherent `impl Type {…}` (no trait) and trait conformance `impl Trait for Type {…}`. Inherent-only narrowing is the embedder's responsibility. |
| data_access | (none in v0) | — |
| error_handling | `try_expression`, `match_expression` | None (both `?` and `match` are counted; callers may apply finer-grained filters in their own extractors) |
| logging | `macro_invocation` where callee matches `^(tracing\|log)::\|^(println\|eprintln\|print\|eprint\|dbg)!` | Natively classified since M33 via `classify_hint`. Examples: `tracing::info!(x)`, `log::debug!(x)`, `println!(x)`. |
| resource_management | `macro_invocation` where callee does NOT match the logging regex above | Logging macros are excluded and routed to `logging` instead. Remaining: `vec!`, `assert!`, `drop!`, `format!`, etc. |
| state_management | `closure_expression` | None |
| type_assertions | `as_expression` | None |

### Python

| Category | Node kinds | Structural constraint |
|---|---|---|
| async_patterns | `await` | None |
| class_hierarchy | `class_definition` | All `class` definitions, including those with no base classes (which are effectively `class Foo(object)` and contribute low entropy). |
| data_access | `call` where callee matches `^(open\(\|requests\.\|httpx\.\|cursor\.\|session\.\|conn\.)` | Natively filtered since M33. Examples: `open(path)`, `cursor.execute(q)`, `requests.get(url)`. Unrecognised calls (e.g. `len(x)`) are dropped. |
| decorators | `decorated_definition` | One instance per decorated function or class definition — wrapper-granularity. Three stacked `@`-lines on one function = one instance. Contrast with TypeScript/JavaScript, which counts one instance per `decorator` line. Added M36.2. |
| error_handling | `try_statement` | None |
| logging | `call` where callee matches `^(logging\.\|print\b)` | Natively classified since M33. Examples: `logging.info(x)`, `print(x)`. |
| resource_management | (none in v0) | — |
| schema_validation | `call` where callee matches `\bField\(|\bconstr\(|\bconint\(` | Pydantic field-constraint call forms only. `class Foo(BaseModel)` counts under `class_hierarchy`, not here. Added M38. |
| state_management | `lambda` | None |
| type_assertions | (none in v0) | — |

### TypeScript / JavaScript

| Category | Node kinds | Structural constraint |
|---|---|---|
| async_patterns | `await_expression`; `call_expression` where callee matches `\.(then\|catch\|finally)\(` | `await_expression` via node-kind; Promise chains via callee-text. Examples: `future.await`, `p.then(r)`, `fetch().catch(e => {})`. |
| class_hierarchy | `class_declaration`, `abstract_class_declaration`, `interface_declaration` | Abstract classes and interfaces always count. Concrete classes count regardless of `extends` / `implements`; entropy survives the broader collection because heritage-free classes have similar structure and contribute low entropy. (JavaScript: only `class_declaration` is emitted; `interface_declaration` and `abstract_class_declaration` are TS-only AST shapes.) |
| collection_pipelines | `call_expression` where callee matches `\.(map\|filter\|reduce\|flatMap\|forEach\|find\|findIndex\|some\|every\|flat)\(` | Member-call pattern — requires a preceding dot. Callee-text cannot distinguish receiver type: `rxObservable.map(fn)`, `new Map().forEach(cb)`, and `array.map(f)` all match; treated as acceptable entropy noise. Natively classified at CALL_DISPATCH slot P10 (M40). |
| data_access | `call_expression` where callee matches `^(fetch\|axios)\b\|\b(query\|read\|write\|get\|post\|put\|delete\|patch)\(\|\b(db\|sql)\.\|\.(query\|read\|write\|fetch)\(` | Natively filtered since M33. Examples: `fetch(url)`, `db.query(sql)`. Unrecognised calls (e.g. `Math.max(a, b)`) are dropped. |
| decorators | `decorator` | Natively classified since M36.1. Examples: `@Injectable()`, `@Component({...})`, `@Entity()`, `@Get('/')`, `@IsString()`. Node-kind only — all decorators count. |
| error_handling | `try_statement` | None |
| framework_hooks | `call_expression` where callee matches `^use[A-Z]` | Natively classified since M35. Examples: `useState(0)`, `useEffect(fn, [])`, `useAuth()`. Second character must be uppercase — `user()` does not match. |
| http_routing | `call_expression` where callee matches `^(app\|router\|fastify\|server\|srv)\.(get\|post\|put\|delete\|patch\|head\|options\|all\|use\|route)\(` | Receiver-allowlist anchored at P7. Examples: `app.get('/u', h)`, `router.post('/user', cb)`, `fastify.route({...})`, `server.use(mw)`. Client calls (`axios.get`, `fetch`) stay `data_access`. NestJS/FastAPI decorator routes stay `decorators`. Added M41. |
| logging | `call_expression` where callee matches `^(console\|logger\|log)\.` | Natively classified since M33. Examples: `console.log(x)`, `logger.info(x)`. |
| null_safety | `optional_chain`; `non_null_expression` (TS only) | `optional_chain`: one instance per node as emitted by the grammar — a nested chain `a?.b?.c` may produce multiple nodes. `non_null_expression`: TypeScript-only; not emitted by the JS adapter. Nullish coalescing (`??`) is deferred. Added M37. |
| resource_management | (none in v0) | — |
| schema_validation | `call_expression` where callee matches `^(z\|yup\|v\|s)\.\w` or `\.safeParse\(` | Namespace-anchored: Zod (`z.`), Yup (`yup.`), Valibot (`v.`), Superstruct (`s.`) + Zod-specific `.safeParse(`. Bare `.string()`/`.object()` on arbitrary receivers are intentionally excluded — no receiver-type info available. `SomeSchema.parse(x)` (no namespace prefix) is a known miss. Natively classified at CALL_DISPATCH slot P4 (M38). |
| state_management | `arrow_function` | None |
| state_store | `call_expression` where callee matches Redux/RTK factories, `^use(Selector\|Dispatch\|Store)\b`, `^create\(`, Jotai/Recoil, MobX, Signals, or Solid createX patterns | All patterns `^`-anchored at callee start. `useSelector`/`useDispatch`/`useStore` match both `state_store` (P5) and `framework_hooks` (P6); state_store wins by precedence. `prisma.user.create(...)` and `document.createElement(...)` are excluded because their callee text does not start with `create(`. Natively classified at CALL_DISPATCH slot P5 (M39). |
| type_assertions | `type_cast_expression`, `as_expression` | None |

### Go / Java

These languages share the common callee-text filter via `classify_hint`. Go and Java `call_expression` nodes are filtered the same way as TypeScript/JavaScript for `data_access` (shared regex table); logging uses per-language regex tables.

| Category | Node kinds | Structural constraint |
|---|---|---|
| class_hierarchy | Java: `class_declaration`, `interface_declaration`. Go: (none in v0 — Go has no class/interface AST shape; the duck-typed interface model does not surface as a `class_hierarchy` declaration. The category exists in the catalog so cross-language reporting is uniform, but it produces zero Go hits.) | Java: same broader-collection caveat as other languages — all declaration kinds are classified regardless of heritage. |
| data_access | `call_expression` where callee matches the shared TS/JS/Go regex (`^(fetch\|axios)\b\|\b(db\|sql)\.` etc.) | Natively filtered since M33. Examples: `db.query(sql)`, `sql.Open(dsn)`. Java `call_expression` returns `false` in v0 — data-access detection is library-shaped and deferred. |
| http_routing | Go: `call_expression` where callee matches `^(http\|mux\|r\|e\|router\|engine\|g\|rg)\.(HandleFunc\|Handle\|GET\|POST\|PUT\|DELETE\|PATCH\|Any\|Group)\(` | Receiver-allowlist anchored at P7. Examples: `http.HandleFunc("/", h)`, `r.GET("/users", h)`, `mux.Handle("/", h)`, `e.POST("/user", h)`. Go uppercase verb names avoid overlap with data_access (lowercase `\bget\(`). Java returns `false` in v0. Added M41. |
| logging | Go: `call_expression` where callee matches `^fmt\.(Print\|Println\|Printf\|Errorf\|Fprint\|Sprint)`. Java: `call_expression` where callee matches `^(System\.(out\|err)\.\|logger\.\|Log\.\|LOG\.)` | Natively classified since M33. Go examples: `fmt.Println(x)`, `fmt.Printf(f, x)`. Java examples: `System.out.println(x)`, `LOG.info(x)`. |

> **Note on per-language node-kind tables:** The v0 tables above are written by hand.
> A future milestone could derive them from the tree-sitter query definitions to eliminate
> this doc/code drift surface. Until then, the [category contract test](../crates/sdivi-core/tests/category_contract.rs)
> is the authoritative drift detector for the category *set*; per-language node-kind
> accuracy relies on manual review.

## Callee-text classification (`classify_hint`)

`sdivi_core::classify_hint(hint, language) -> Vec<String>` provides a
higher-precision classifier that inspects both the `node_kind` and `hint.text` (the
truncated source text of the node). Foreign extractors should prefer `classify_hint`
over hand-rolled callee filters — the regex tables below are part of the canonical
contract and are versioned with `snapshot_version "1.0"`.

**As of M33, the regex tables are load-bearing for native pipeline output, not just
embedder convenience.** `Pipeline::snapshot` now calls `classify_hint` instead of
`category_for_node_kind`. Per-category instance counts shift on upgrade — see
`MIGRATION_NOTES.md` for the M33 migration story and a worked example.

### `data_access::matches_callee(text, language)`

| Language | Pattern | Examples matched |
|---|---|---|
| TypeScript / JavaScript / Go | `^(fetch\|axios)\b` | `fetch("/api")`, `axios.get(url)` |
| TypeScript / JavaScript / Go | `\b(query\|read\|write\|get\|post\|put\|delete\|patch)\(` | `db.query(sql)`, `get(url)` |
| TypeScript / JavaScript / Go | `\b(db\|sql)\.` | `db.execute(sql)`, `sql.Open(dsn)` |
| TypeScript / JavaScript / Go | `\.(query\|read\|write\|fetch)\(` | `client.read(buf)`, `.fetch(url)` |
| Python | `^(open\(\|requests\.\|httpx\.\|cursor\.\|session\.\|conn\.)` | `open(path)`, `cursor.execute(q)` |
| Rust, Java | (none in v0) | — |

**Worked example (TypeScript):** `fetch("/api/users")` → `["data_access"]`

### `logging::matches_callee(text, language)`

| Language | Pattern | Examples matched |
|---|---|---|
| TypeScript / JavaScript | `^(console\|logger\|log)\.` | `console.log(x)`, `logger.info(x)` |
| Python | `^(logging\.\|print\b)` | `logging.info(x)`, `print(x)` |
| Go | `^fmt\.(Print\|Println\|Printf\|Errorf\|Fprint\|Sprint)` | `fmt.Println(x)`, `fmt.Printf(f, x)` |
| Rust | `^(tracing\|log)::\|^(println\|eprintln\|print\|eprint\|dbg)!` | `tracing::info!(x)`, `println!(x)` |
| Java | `^(System\.(out\|err)\.\|logger\.\|Log\.\|LOG\.)` | `System.out.println(x)`, `LOG.info(x)` |

**Worked example (Rust):** `tracing::info!("request")` → `["logging"]`

### `async_patterns::matches_callee(text, language)`

| Language | Pattern | Examples matched |
|---|---|---|
| TypeScript / JavaScript | `\.(then\|catch\|finally)\(` | `p.then(r)`, `fetch().catch(e => {})` |
| All others | (none) | — |

**Worked example (TypeScript):** `promise.then(resolve)` → `["async_patterns"]`

### `schema_validation::matches_callee(text, language)`

| Language | Pattern | Examples matched | Deliberately NOT matched |
|---|---|---|---|
| TypeScript / JavaScript | `^(z\|yup\|v\|s)\.\w` | `z.object({})`, `yup.string()`, `v.pipe(...)`, `s.object(...)` | bare `.string()`/`.array()` on arbitrary receivers |
| TypeScript / JavaScript | `\.safeParse\(` | `UserSchema.safeParse(x)` | bare `.parse(` (collides with date/arg parsers) |
| Python | `\bField\(` | `Field(default=0)`, `Field(...)` | — |
| Python | `\bconstr\(` | `constr(min_length=1)` | — |
| Python | `\bconint\(` | `conint(gt=0)` | — |
| All others | (none) | — | — |

**Worked example (TypeScript):** `z.object({ name: z.string() })` → `["schema_validation"]`

**Known recall gap:** `SomeSchema.parse(x)` where the receiver name is arbitrary is not captured — receiver-type info is outside the v0 node-kind model. Receiver-type inference would require a separate analysis pass; treat as out of scope, not a regex tweak.

**Pydantic class coverage:** `class Foo(BaseModel)` is a `class_definition` and already counted under `class_hierarchy` (M6). Python coverage here is intentionally partial: only call forms (`Field(...)`, `constr(...)`, `conint(...)`) are classified. class-validator decorators (`@IsString()`) belong to `decorators` (M36.1/M36.2).

### `state_store::matches_callee(text, language)`

| Language | Pattern | Examples matched | Deliberately NOT matched |
|---|---|---|---|
| TypeScript / JavaScript | `^(createSlice\|configureStore\|createStore\|combineReducers\|createAsyncThunk\|createReducer\|createAction\|createSignal\|createEffect\|createMemo\|createResource)\(` | `createSlice({})`, `createSignal(0)` | — |
| TypeScript / JavaScript | `^use(Selector\|Dispatch\|Store)\b` | `useSelector(s => s.x)`, `useDispatch()` | `useAuth()`, `useState()` (fall through to `framework_hooks`) |
| TypeScript / JavaScript | `^create\(` | `create((set) => ({}))` (Zustand) | `prisma.user.create(data)`, `document.createElement(...)` |
| TypeScript / JavaScript | `^(atom\|selector\|atomFamily\|selectorFamily)\(` | `atom(0)`, `selector({...})` | — |
| TypeScript / JavaScript | `^(observable\|action\|computed\|makeObservable\|makeAutoObservable\|runInAction\|signal\|effect\|batch)\(` | `makeAutoObservable(this)`, `signal(0)` | — |
| All others | (none) | — | — |

**Worked example (TypeScript):** `createSlice({name: 'users', ...})` → `["state_store"]`

**Worked example (TypeScript):** `useSelector(s => s.user)` → `["state_store"]` (P5 beats `framework_hooks` P6)

**`^`-anchor rationale:** State-store factory calls are imported and called bare (`create(...)`, `atom(0)`, `signal(0)`). Anchoring at callee start captures bare calls while excluding member-access calls (`prisma.user.create(data)` starts with `prisma`, not `create`). A residual false positive exists for a local function named `create(x)` or `effect(x)` unrelated to any store — treated as entropy noise at codebase scale.

**Open question (TanStack Query / SWR):** `useQuery`, `useMutation`, `useSWR` blur "state" and "data-fetching". Their home is deferred to a follow-up milestone. Until then they fall through to `framework_hooks`.

### `http_routing::matches_callee(text, language)`

| Language | Pattern | Examples matched | Stays `data_access` |
|---|---|---|---|
| TypeScript / JavaScript | `^(app\|router\|fastify\|server\|srv)\.(get\|post\|put\|delete\|patch\|head\|options\|all\|use\|route)\(` | `app.get('/u', h)`, `router.post('/user', cb)`, `fastify.route({...})`, `server.use(mw)`, `srv.all('*', h)` | `axios.get(url)`, `fetch(url)`, `client.get(url)`, `cache.get(k)` |
| Go | `^(http\|mux\|r\|e\|router\|engine\|g\|rg)\.(HandleFunc\|Handle\|GET\|POST\|PUT\|DELETE\|PATCH\|Any\|Group)\(` | `http.HandleFunc("/", h)`, `r.GET("/u", h)`, `mux.Handle("/", h)`, `e.POST("/u", h)`, `engine.Group("/api")` | `db.Query(sql)`, `sql.Open(dsn)` |
| Python | `\.add_url_rule\(` | `app.add_url_rule('/u', view_func=h)` | — (FastAPI/Flask decorator routes → `decorators`) |
| All others | (none) | — | — |

**Worked example (TypeScript):** `app.get('/users', handler)` → `["http_routing"]`

**Worked example (TypeScript — client, stays data_access):** `axios.get(url)` → `["data_access"]`

**Receiver-allowlist rationale:** The allowlist (`app`, `router`, `fastify`, `server`, `srv`; Go adds `r`, `e`, `mux`, `engine`, `g`, `rg`) is the precision mechanism. A client GET (`axios.get`, receiver `axios`) is outside every allowlist and correctly remains `data_access`. An idiosyncratically-named server (`const api = express(); api.get(...)`) will be missed — documented limitation.

**NestJS / FastAPI distinction:** Decorator-style routes (`@Get('/')`, `@app.get(...)`) are `decorator` / `decorated_definition` nodes, classified under `decorators` (M36.1/M36.2). They are counted once, in `decorators`, not here.

### `collection_pipelines::matches_callee(text, language)`

| Language | Pattern | Examples matched | Deliberately NOT matched |
|---|---|---|---|
| TypeScript / JavaScript / Go / Java | `\.(map\|filter\|reduce\|flatMap\|forEach\|find\|findIndex\|some\|every\|flat)\(` | `xs.map(f)`, `xs.filter(p).reduce(g, 0)`, `items.forEach(cb)` | bare `map(f)` (no dot), `db.query(sql)` (data_access), `promise.then(r)` (async_patterns) |
| All others | (none) | — | — |

**Worked example (TypeScript):** `xs.filter(isActive).map(toDto)` → `["collection_pipelines"]`

**Receiver-type noise:** Callee-text cannot distinguish an array `.map` from `rxObservable.map(fn)` (RxJS), `new Map().forEach(cb)` (ES6 Map), or `domNodeList.forEach(cb)` (DOM). This is intentional — the signal is the functional-iteration population at codebase scale, not the receiver type of each call. Receiver-type inference would require a type-info pass outside the v0 model.

**Pipe/compose seeds forward:** `pipe(...)`, `compose(...)`, `flow(...)` from lodash/fp-ts/Ramda are the same idiom family and could extend this regex in a future milestone.

### `framework_hooks::matches_callee(text, language)`

| Language | Pattern | Examples matched |
|---|---|---|
| TypeScript / JavaScript | `^use[A-Z]` | `useState(0)`, `useEffect(fn, [])`, `useAuth()`, `useStore()` |
| All others | (none) | — |

**Worked example (TypeScript):** `useState(0)` → `["framework_hooks"]`

**Note:** The second character must be uppercase — `user()`, `useful()` do not match. The anchor `^` prevents mid-identifier matches.

### `resource_management::excludes_callee(text, language)`

This function is **inverted**: returns `true` when a `macro_invocation` should fall
through to `logging` instead of staying in `resource_management`.

| Language | Pattern | Examples excluded |
|---|---|---|
| Rust | `^(tracing\|log)::\|^(println\|eprintln\|print\|eprint\|dbg)!` | `tracing::info!`, `println!` |
| All others | (none) | — |

**Worked example (Rust):** `vec![1, 2, 3]` macro invocation → `["resource_management"]`;
`tracing::info!("x")` macro invocation → `["logging"]`.

### Dispatch order in `classify_hint`

`classify_hint`'s `call_expression`/`call` arm iterates the `CALL_DISPATCH` registry
(`crates/sdivi-patterns/src/queries/mod.rs`). **First match wins.** The order below
is the contract — future milestones insert at their named slot, never append.

#### Canonical precedence table

| Slot | Category | Active | Representative regex / pattern |
|---|---|---|---|
| P1 | `async_patterns` | M34 | `\.(then\|catch\|finally)\(` |
| P2 | `testing` | M42 | `^(describe\|it\|test\|expect)\(`, `^jest\.` |
| P3 | `serialization` | M43 | `^JSON\.(parse\|stringify)\(`, `^json\.(Marshal\|Unmarshal)\(` |
| P4 | `schema_validation` | M38 | `^(z\|yup\|v\|s)\.\w`, `\.safeParse\(`, `\bBaseModel\b` |
| P5 | `state_store` | M39 | redux/zustand/jotai factories; `^use(Selector\|Dispatch\|Store)\b` |
| P6 | `framework_hooks` | M35 | `^use[A-Z]` |
| P7 | `http_routing` | **M41** | `^(app\|router\|fastify\|server\|srv)\.(get\|post\|…)\(` |
| P8 | `logging` | M34 | `^(console\|logger\|log)\.`, `^fmt\.Print`, `^(tracing\|log)::` |
| P9 | `data_access` | M34 | `^(fetch\|axios)\b`, `\b(db\|sql)\.`, `cursor\.`, `requests\.` |
| P10 | `collection_pipelines` | M40 | `\.(map\|filter\|reduce\|flatMap\|forEach\|find\|findIndex\|some\|every\|flat)\(` |
| P11 | `concurrency` | M44 | `^Promise\.(all\|allSettled\|race\|any)\(`, `^asyncio\.gather\(` |

P1, P4, P5, P6, P7, P8, P9, and P10 are active at M41. The `decorators` and `null_safety` categories are
node-kind-only and do not appear in `CALL_DISPATCH` — they are classified via
`category_for_node_kind` in the `other =>` arm of `classify_hint`. All other slots are
reserved placeholders.

#### KNOWN_OVERLAPS policy

When a callee string legitimately matches two categories' regexes, the first-match
winner is correct by construction. The overlap must be documented in the
`KNOWN_OVERLAPS` table in `crates/sdivi-patterns/tests/dispatch_disjointness.rs`.

Documented overlaps at M41 (P1/P4/P5/P6/P7/P8/P9/P10 active):

| Callee | Language | Winner | Loser | Rationale |
|---|---|---|---|---|
| `fetch(url).catch(err => {})` | javascript | `async_patterns` | `data_access` | Chained-fetch outer node matches both `\.(catch)\(` (P1) and `^fetch\b` (P9); P1 wins by precedence |
| `useSelector(s => s.user)` | typescript | `state_store` | `framework_hooks` | Matches both `^use(Selector\|Dispatch\|Store)\b` (P5) and `^use[A-Z]` (P6); more specific wins |
| `useDispatch()` | typescript | `state_store` | `framework_hooks` | Same as above |
| `useStore()` | javascript | `state_store` | `framework_hooks` | Same as above |
| `app.get('/u', h)` | typescript | `http_routing` | `data_access` | Receiver `app` in allowlist (P7); `\b(get)\(` also matches data_access (P9); P7 wins |
| `router.post('/user', cb)` | typescript | `http_routing` | `data_access` | Same mechanism — `router` in allowlist (P7); `\b(post)\(` matches data_access (P9) |

Future overlaps introduced by M42–M44:
- `Promise.all([]).then(cb)` outer node → **async_patterns** (P1) wins; bare inner `Promise.all(…)` resolves to `concurrency` (P11).

#### `macro_invocation` arm

For `macro_invocation`:
- Logging macros (Rust only) → `["logging"]`
- All others → `["resource_management"]`

All other node kinds fall through to `category_for_node_kind`.

### Regex change log

First defined in M32 for `snapshot_version "1.0"`. Changing or narrowing a regex
is a behavioural break requiring a `MIGRATION_NOTES.md` entry. Broadening (adding
new shapes) is additive.

## Normalization rules

Pattern fingerprints are computed by `sdivi_core::normalize_and_hash`. The algorithm is:

1. **Input**: a `node_kind: &str` and an ordered `children: &[NormalizeNode]` slice.
2. **Leaf node** (`children` is empty): `blake3::keyed_hash(FINGERPRINT_KEY, node_kind.as_bytes())` — byte-identical to `fingerprint_node_kind(node_kind)`.
3. **Internal node**: input bytes = `node_kind.as_bytes()` + `0x00` + for each child: `0x01` + 32 child-digest bytes.
4. **Key**: `FINGERPRINT_KEY` — a 32-byte constant defined in `sdivi_patterns::fingerprint::FINGERPRINT_KEY` and re-exported from `sdivi_core::FINGERPRINT_KEY`. The key is **fixed for all `snapshot_version "1.0"` output**. Changing the key invalidates all existing snapshot fingerprints.

### Embedder responsibilities

An embedder that supplies `PatternInstanceInput` values must:

1. Use category names **verbatim** as returned by `list_categories()`. The comparison in `compute_pattern_metrics` is case-sensitive.
2. Compute fingerprints via `normalize_and_hash(node_kind, children)` (Rust) or the WASM export `normalize_and_hash(nodeKind, children)`. Do not implement a custom fingerprint algorithm.
3. When calling `normalize_and_hash`, pass the tree-sitter `node_kind` string and, if available, the ordered child subtree. For v0 language adapters, children is always empty — leaf-level fingerprints only.
4. The fingerprint must be a 64-character lowercase hex string as returned by `normalize_and_hash`.
5. **`data_access` is callee-filtered since M33.** Only `call_expression`/`call` nodes whose callee text matches the per-language data-access regex are classified here. Embedders that supply `PatternInstanceInput { category: "data_access", … }` directly continue to work — their instances merge with natively classified ones. Embedders that want custom callee filters should apply them before calling `compute_pattern_metrics`.
6. **As of M33, the `logging` category is natively classified by the pipeline via `classify_hint`.** `sdivi_patterns::queries::category_for_node_kind` still never returns `Some("logging")` — that sentinel is unchanged — but the native pipeline now calls `classify_hint`, which routes matching callees to `logging`. Embedders that pass `PatternInstanceInput { category: "logging" }` directly will continue to round-trip — their instances merge with the natively-classified ones in `compute_pattern_metrics` output. Embedders that previously hand-rolled their own logging filter should consider switching to `classify_hint` (M32) to stay aligned with the canonical regex set.
7. **As of M35, the `framework_hooks` category is natively classified for TypeScript and JavaScript** via `classify_hint` callee-text inspection (`^use[A-Z]` regex). Hook calls that were previously unrecognised (dropped to `[]`) are now counted in the `framework_hooks` bucket. On the first post-M35 snapshot of a TS/JS repo, `framework_hooks` transitions from zero to non-zero. This is a count-introduction event; see `MIGRATION_NOTES.md` for details.
8. **As of M36.1, the `decorators` category is natively classified for TypeScript and JavaScript** via the `decorator` tree-sitter node kind. `@Injectable()`, `@Component({...})`, `@Entity()`, `@Get('/')`, and any other decorator node are counted. On the first post-M36.1 snapshot of a TS/JS repo with decorators, the `decorators` bucket transitions from zero to non-zero — a count-introduction event; see `MIGRATION_NOTES.md`. **As of M36.2, Python is also supported** via the `decorated_definition` node kind (`@dataclass`, `@property`, `@app.get(...)`, `@pytest.fixture`, etc.). Count semantics differ: Python counts one instance per decorated definition (wrapper-granularity); TypeScript/JavaScript count one per decorator line; see `MIGRATION_NOTES.md`.
9. **As of M37, the `null_safety` category is natively classified for TypeScript and JavaScript** via `optional_chain` and `non_null_expression` node kinds. On the first post-M37 snapshot of a TS/JS repo using optional chaining or non-null assertions, the `null_safety` bucket transitions from zero to non-zero — a count-introduction event; see `MIGRATION_NOTES.md`. Nullish coalescing (`??`) is deferred — it requires operator-field inspection beyond the v0 node-kind model.
10. **As of M38, the `schema_validation` category is natively classified for TypeScript, JavaScript, and Python** via callee-text inspection at CALL_DISPATCH slot P4. Zod (`z.*`), Yup (`yup.*`), Valibot (`v.*`), Superstruct (`s.*`), `.safeParse(`, and Pydantic field-constraint calls (`Field(...)`, `constr(...)`, `conint(...)`) are now counted in `schema_validation`. On the first post-M38 snapshot of a repo using these libraries, the `schema_validation` bucket transitions from zero to non-zero — a count-introduction event; see `MIGRATION_NOTES.md`.
11. **As of M39, the `state_store` category is natively classified for TypeScript and JavaScript** via callee-text inspection at CALL_DISPATCH slot P5. Redux/RTK factories (`createSlice`, `configureStore`, etc.), React-Redux hooks (`useSelector`, `useDispatch`, `useStore`), Zustand (`create(...)`), Jotai/Recoil (`atom`, `selector`), MobX (`observable`, `makeAutoObservable`, etc.), Signals (`signal`, `computed`, `effect`), and Solid (`createSignal`, `createStore`, etc.) are now counted in `state_store`. **Precedence reassignment:** `useSelector`, `useDispatch`, and `useStore` previously resolved to `framework_hooks` (P6); they now resolve to `state_store` (P5). This is a count shift between two new-in-M35/M39 categories — counts move from `framework_hooks` to `state_store`. See `MIGRATION_NOTES.md` for the canonical precedence-reassignment example.
12. **As of M40, the `collection_pipelines` category is natively classified** via member-call callee-text at CALL_DISPATCH slot P10 (broadest member-call category — all more-specific categories resolve first). `.map`, `.filter`, `.reduce`, `.flatMap`, `.forEach`, `.find`, `.findIndex`, `.some`, `.every`, `.flat` on any receiver are now counted in `collection_pipelines` for TypeScript and JavaScript (and Go/Java where these method names appear). Callee-text cannot distinguish the receiver type — `rxObservable.map(fn)`, `new Map().forEach(cb)`, and `array.map(f)` all match; treated as acceptable entropy noise. Bare calls without a dot prefix (`map(f)`) are intentionally not matched. On the first post-M40 snapshot of a TS/JS repo using these methods, `collection_pipelines` transitions from zero to non-zero — a count-introduction event; see `MIGRATION_NOTES.md`.
13. **As of M41, the `http_routing` category is natively classified** via receiver-allowlist-anchored callee-text at CALL_DISPATCH slot P7 (above `logging` P8 and `data_access` P9). Express/Koa/Fastify (`app.get`, `router.post`, `fastify.route`, `server.use`), Go net/http + Gin/Echo/Gorilla (`http.HandleFunc`, `r.GET`, `mux.Handle`), and Flask/FastAPI imperative registration (`app.add_url_rule`) are now counted in `http_routing`. **Precedence note:** `app.get(...)` / `router.post(...)` previously matched `data_access` (P9) via the `\b(get|post)\(` verb regex; they now resolve to `http_routing` (P7) — a count shift between an existing and a new category. `axios.get(url)` / `client.get(url)` stay in `data_access` because their receiver is outside the allowlist. NestJS and FastAPI decorator routes (`@Get('/')`, `@app.get(...)`) are `decorator`/`decorated_definition` nodes and remain in `decorators`. See `MIGRATION_NOTES.md` for the worked before/after.
14. **The `class_hierarchy` category in `snapshot_version "1.0"` is wired natively but classified broadly** — every declaration of the listed node kinds is included regardless of heritage. Embedders that want heritage-only precision (e.g. only classes with an `extends` clause, only `impl Trait for …` blocks) should filter `PatternInstanceInput` on their side before passing to `compute_pattern_metrics`. Entropy-based divergence signals remain meaningful under the broader collection because hierarchy-free declarations contribute low structural variance — the signal is the variance introduced by hierarchical declarations, not the absolute count.

Cross-runtime determinism: the WASM `normalize_and_hash` produces **bit-identical** output to the native Rust pipeline for the same input. See `docs/determinism.md` for the full guarantee.

## Runtime discovery (recommended)

Call `list_categories()` at startup rather than hard-coding category names:

```rust
use sdivi_core::list_categories;

let catalog = list_categories();
for cat in &catalog.categories {
    println!("{}: {}", cat.name, cat.description);
}
```

From WASM / TypeScript:

```ts
import init, { list_categories } from '@geoffgodwin/sdivi-wasm';

await init();
const catalog = list_categories();
console.log(catalog.schema_version); // "1.0"
for (const cat of catalog.categories) {
    console.log(cat.name, '-', cat.description);
}
```
