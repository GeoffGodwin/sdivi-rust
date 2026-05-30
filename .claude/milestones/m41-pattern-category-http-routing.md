
#### Milestone 41: Pattern Category — `http_routing`

<!-- milestone-meta
id: "41"
status: "planned"
-->

**Scope:** Add the `http_routing` category for server-side route/endpoint
declarations. TS/JS: Express/Koa/Fastify (`app.get`, `router.post`,
`fastify.route`), Next.js route handlers, Hono. NestJS routes are decorator-based
and are covered by `decorators` (M36.1) — cross-referenced, not duplicated.
Python: Flask/FastAPI route decorators (covered by `decorators` M36.2) plus
`app.add_url_rule`. Go: `http.HandleFunc`, `mux.HandleFunc`, Gin `r.GET/POST`.
Callee-text on `call_expression`; depends on M34 and M36.1.

**Why this milestone exists:** Endpoint declaration style (decorator routing vs
imperative `app.get`, router composition) is a high-signal convention in backend
codebases. A dedicated bucket makes API-surface drift visible.

**Deliverables:**
- Create `crates/sdivi-patterns/src/queries/http_routing.rs` with per-language regex.
- Register in `ALL_CATEGORIES`, M34 `CALL_DISPATCH` at **slot P7 (above
  `data_access` P9)** — `app.get`/`router.post` would otherwise match the
  data-access `\b(get\|post\|...)\(` regex; this is the headline disambiguation.
  Add `CATALOG_ENTRIES`.
- Update `docs/pattern-categories.md`.

**Detection (finalized — receiver-allowlist anchored so server route defs are
peeled off but client calls stay `data_access`):**
| Language | Pattern | Examples matched | Stays `data_access` |
|---|---|---|---|
| TS / JS | `^(app\|router\|fastify\|server\|srv)\.(get\|post\|put\|delete\|patch\|head\|options\|all\|use\|route)\(` | `app.get('/u', h)`, `router.post(...)`, `app.use(mw)` | `axios.get(url)`, `fetch(...)`, `client.get(...)`, `cache.get(k)` |
| Go | `^(http\|mux\|r\|e\|router\|engine\|g\|rg)\.(HandleFunc\|Handle\|GET\|POST\|PUT\|DELETE\|PATCH\|Any\|Group)\(` | `http.HandleFunc(...)`, `r.GET("/u", h)`, `e.POST(...)` | `db.Query(...)`, `sql.Open(...)` |
| Python | `\.add_url_rule\(` | `app.add_url_rule(...)` | — (FastAPI/Flask decorator routes → `decorators`, M36.2) |

**The receiver allowlist is the precision mechanism.** Routing is detected only
when the call's receiver is a known server/router handle (`app`, `router`,
`fastify`, `server`, `srv`; Go adds `r`, `e`, `mux`, `engine`, `g`, `rg`). A
client GET (`axios.get`, receiver `axios`) is not in the list and correctly remains
`data_access`. An idiosyncratically-named server (`const api = express(); api.get`)
will be missed — documented limitation.

**Migration Impact:** Additive; `list_categories()` +1. **Draws from
`data_access`:** `app.get(...)`/`router.post(...)` currently match the data-access
HTTP-verb regex and are classified as `data_access`. On upgrade they reassign to
`http_routing` — a count shift between an existing and a new category. This is the
most consequential migration in the TS/JS track; document it prominently in
`MIGRATION_NOTES.md` with a worked before/after. `snapshot_version` stays `"1.0"`.

**Files to create or modify:**
- **Create:** `crates/sdivi-patterns/src/queries/http_routing.rs`.
- **Modify:** `crates/sdivi-patterns/src/queries/mod.rs` (registry order!),
  `crates/sdivi-core/src/categories.rs`.
- **Modify:** `docs/pattern-categories.md`, `MIGRATION_NOTES.md`, `CHANGELOG.md`.

**Acceptance criteria:**
- `app.get('/u', h)` → `["http_routing"]` (NOT `data_access`).
- `axios.get(url)` → `["data_access"]` (client GET stays data-access).
- `category_contract.rs`, WASM count test, clippy/fmt/doc gates green.

**Tests:**
- Unit: server-route positives across Express/Gin; client-fetch negatives.
- Disjointness corpus: `app.get` vs `axios.get` documented split, both asserted.

**Watch For:**
- **The `app.get` vs `axios.get` problem is the crux.** Both are `.get(`. Disambiguate
  on the receiver token (`app`/`router`/`fastify`/`r`/`mux`/`e` ⇒ routing;
  `axios`/`fetch`/`client`/`http` client ⇒ data-access). Encode receiver lists
  explicitly; accept that an idiosyncratically-named receiver (`const server = ...;
  server.get`) may be miscategorised — document the heuristic's limits.
- **NestJS/FastAPI routes are decorators, not calls** — ensure they are counted once
  (under `decorators`), and that M36.1's decorator collection does not also surface
  the inner `@Get('/')` call as `http_routing`. Cross-reference both milestones.

**Seeds Forward:**
- GraphQL resolvers, gRPC service methods, and tRPC routers are adjacent
  "endpoint declaration" idioms that could warrant their own category or extend
  this one. Out of scope; note the open question.
