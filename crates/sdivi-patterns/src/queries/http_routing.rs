//! Callee-text classification for server-side HTTP route/endpoint declarations.
//!
//! Detects route-registration calls in Express/Koa/Fastify (TypeScript/JavaScript),
//! Hono, Next.js route handlers, `http.HandleFunc`/Gin (Go), and `add_url_rule`
//! (Python Flask/FastAPI imperative registration). Detection is anchored on the
//! receiver token so that client-side HTTP calls (`axios.get`, `fetch`) stay in
//! `data_access`.
//!
//! ## Receiver-allowlist precision
//!
//! The TS/JS regex matches only when the call receiver is a known server/router
//! handle: `app`, `router`, `fastify`, `server`, `srv`. Go adds `http`, `mux`,
//! `r`, `e`, `engine`, `g`, `rg`. A client call like `axios.get(url)` (receiver
//! `axios`) is outside every allowlist and correctly stays in `data_access`.
//!
//! **Documented limitation:** An idiosyncratically-named server variable
//! (`const api = express(); api.get(...)`) will not be detected — receiver-type
//! inference would require a type-info pass outside the v0 node-kind model.
//!
//! ## NestJS / FastAPI distinction
//!
//! NestJS route decorators (`@Get('/')`, `@Post(...)`) and FastAPI route
//! decorators (`@app.get(...)`, `@app.post(...)`) are `decorator` / `decorated_definition`
//! nodes classified under `decorators` (M36.1/M36.2). They are intentionally **not**
//! duplicated here — each route is counted once, under `decorators`.
//!
//! ## CALL_DISPATCH slot
//!
//! Registered at P7 in `CALL_DISPATCH` — above `logging` (P8) and `data_access` (P9)
//! so that `app.get(...)` / `router.post(...)` are peeled off before the broad
//! data-access `\b(get|post|...)\(` regex matches them.
//!
//! ## Python `add_url_rule`
//!
//! Flask/FastAPI also supports imperative `app.add_url_rule('/path', view_func=h)`.
//! This is not covered by `decorators` because it is a call, not a decorator.
//! Matched by a member-call regex anchored on `.add_url_rule(`.
//!
//! ## Seeds forward
//!
//! GraphQL resolvers, gRPC service methods, and tRPC routers are adjacent
//! "endpoint declaration" idioms. Out of scope for M41; a future milestone could
//! extend this category or introduce a sibling.

use std::sync::LazyLock;

use regex::Regex;

/// Tree-sitter node kinds for HTTP routing patterns.
///
/// Empty — this category is detected entirely via callee-text inspection in
/// [`matches_callee`]. `call_expression` nodes are already collected by the
/// adapters; classification happens in `classify_hint`'s `CALL_DISPATCH` loop
/// at slot P7.
pub const NODE_KINDS: &[&str] = &[];

// TypeScript / JavaScript — receiver-allowlist anchored.
// Receiver: app | router | fastify | server | srv
// Methods:  get | post | put | delete | patch | head | options | all | use | route
//
// Disjointness from data_access:
//   axios.get(url)   → receiver `axios` is NOT in the allowlist → falls through to P9
//   client.get(url)  → receiver `client` is NOT in the allowlist → falls through to P9
//   app.get('/u', h) → receiver `app` IS in the allowlist → caught here at P7
static TS_JS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(app|router|fastify|server|srv)\.(get|post|put|delete|patch|head|options|all|use|route)\(",
    )
    .expect("http_routing TS/JS regex is valid")
});

// Go — uppercase HTTP-verb method names on known router/engine receivers.
// Receiver: http | mux | r | e | router | engine | g | rg
// Methods:  HandleFunc | Handle | GET | POST | PUT | DELETE | PATCH | Any | Group
static GO_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(http|mux|r|e|router|engine|g|rg)\.(HandleFunc|Handle|GET|POST|PUT|DELETE|PATCH|Any|Group)\(",
    )
    .expect("http_routing Go regex is valid")
});

// Python — Flask/FastAPI imperative URL registration.
// `app.add_url_rule('/path', view_func=handler)` — member-call anchored on the
// method name; any receiver matches (typically `app` or a Blueprint).
// FastAPI/Flask decorator routes are `decorated_definition` → `decorators` (M36.2).
static PYTHON_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\.add_url_rule\(").expect("http_routing Python regex is valid"));

/// Return `true` when `text` looks like a server-side route/endpoint declaration.
///
/// Detection is receiver-allowlist anchored so that client HTTP calls
/// (`axios.get`, `fetch`) stay in `data_access`. See module doc for the full
/// allowlist and the NestJS / FastAPI decorator distinction.
///
/// # Examples
///
/// ```rust
/// use sdivi_patterns::queries::http_routing::matches_callee;
///
/// // TypeScript/JavaScript — Express/Fastify/Koa
/// assert!(matches_callee("app.get('/users', handler)", "typescript"));
/// assert!(matches_callee("router.post('/user', cb)", "javascript"));
/// assert!(matches_callee("fastify.route({ method: 'GET', ... })", "typescript"));
/// assert!(matches_callee("server.use(middleware)", "typescript"));
/// assert!(matches_callee("srv.all('*', h)", "javascript"));
///
/// // Go — net/http + Gin/Echo/Gorilla
/// assert!(matches_callee("http.HandleFunc(\"/\", h)", "go"));
/// assert!(matches_callee("r.GET(\"/users\", h)", "go"));
/// assert!(matches_callee("mux.Handle(\"/\", h)", "go"));
/// assert!(matches_callee("e.POST(\"/user\", h)", "go"));
///
/// // Python — Flask/FastAPI imperative
/// assert!(matches_callee("app.add_url_rule('/users', view_func=h)", "python"));
///
/// // Client HTTP calls stay in data_access — NOT matched
/// assert!(!matches_callee("axios.get(url)", "typescript"));
/// assert!(!matches_callee("client.get(url)", "typescript"));
/// assert!(!matches_callee("cache.get(key)", "typescript"));
/// assert!(!matches_callee("db.query(sql)", "go"));
/// assert!(!matches_callee("requests.get(url)", "python"));
/// ```
pub fn matches_callee(text: &str, language: &str) -> bool {
    match language {
        "typescript" | "javascript" => TS_JS_RE.is_match(text),
        "go" => GO_RE.is_match(text),
        "python" => PYTHON_RE.is_match(text),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_get_matches_ts() {
        assert!(matches_callee("app.get('/users', handler)", "typescript"));
    }

    #[test]
    fn router_post_matches_js() {
        assert!(matches_callee("router.post('/user', cb)", "javascript"));
    }

    #[test]
    fn fastify_route_matches() {
        assert!(matches_callee(
            "fastify.route({ method: 'GET' })",
            "typescript"
        ));
    }

    #[test]
    fn server_use_matches() {
        assert!(matches_callee("server.use(middleware)", "typescript"));
    }

    #[test]
    fn srv_all_matches() {
        assert!(matches_callee("srv.all('*', h)", "javascript"));
    }

    #[test]
    fn app_delete_matches() {
        assert!(matches_callee("app.delete('/user/:id', h)", "typescript"));
    }

    #[test]
    fn app_put_matches() {
        assert!(matches_callee("app.put('/user', update)", "typescript"));
    }

    #[test]
    fn app_patch_matches() {
        assert!(matches_callee("app.patch('/user/:id', h)", "typescript"));
    }

    #[test]
    fn app_head_matches() {
        assert!(matches_callee("app.head('/ping', h)", "javascript"));
    }

    #[test]
    fn app_options_matches() {
        assert!(matches_callee("app.options('/api', h)", "typescript"));
    }

    #[test]
    fn go_http_handle_func_matches() {
        assert!(matches_callee("http.HandleFunc(\"/\", h)", "go"));
    }

    #[test]
    fn go_gin_r_get_matches() {
        assert!(matches_callee("r.GET(\"/users\", h)", "go"));
    }

    #[test]
    fn go_echo_e_post_matches() {
        assert!(matches_callee("e.POST(\"/user\", h)", "go"));
    }

    #[test]
    fn go_mux_handle_matches() {
        assert!(matches_callee("mux.Handle(\"/\", h)", "go"));
    }

    #[test]
    fn go_engine_group_matches() {
        assert!(matches_callee("engine.Group(\"/api\")", "go"));
    }

    #[test]
    fn python_add_url_rule_matches() {
        assert!(matches_callee(
            "app.add_url_rule('/u', view_func=h)",
            "python"
        ));
    }

    #[test]
    fn axios_get_does_not_match() {
        // client GET stays data_access — receiver `axios` not in allowlist
        assert!(!matches_callee("axios.get(url)", "typescript"));
    }

    #[test]
    fn client_get_does_not_match() {
        assert!(!matches_callee("client.get(url)", "typescript"));
    }

    #[test]
    fn cache_get_does_not_match() {
        assert!(!matches_callee("cache.get(key)", "typescript"));
    }

    #[test]
    fn fetch_does_not_match() {
        assert!(!matches_callee("fetch(\"/api\")", "typescript"));
    }

    #[test]
    fn go_db_query_does_not_match() {
        assert!(!matches_callee("db.Query(sql)", "go"));
    }

    #[test]
    fn python_requests_get_does_not_match() {
        assert!(!matches_callee("requests.get(url)", "python"));
    }

    #[test]
    fn rust_returns_false() {
        assert!(!matches_callee("app.get('/u', h)", "rust"));
    }

    #[test]
    fn java_returns_false() {
        assert!(!matches_callee("app.get('/u', h)", "java"));
    }

    #[test]
    fn node_kinds_is_empty() {
        assert!(NODE_KINDS.is_empty());
    }
}
