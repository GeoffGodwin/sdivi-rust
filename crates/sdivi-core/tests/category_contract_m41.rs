//! M41 acceptance criterion tests.
//!
//! Verifies that:
//! - `app.get('/u', h)` → `["http_routing"]` (NOT `data_access`)
//! - `axios.get(url)` → `["data_access"]` (client GET stays data_access)
//! - `list_categories()` includes `http_routing`

use sdivi_patterns::queries::classify_hint;
use sdivi_patterns::PatternHintInput;

fn hint(text: &str) -> PatternHintInput {
    PatternHintInput {
        node_kind: "call_expression".to_string(),
        text: text.to_string(),
    }
}

// ── Headline: app.get vs axios.get disambiguation ─────────────────────────────

#[test]
fn app_get_is_http_routing_not_data_access() {
    assert_eq!(
        classify_hint(&hint("app.get('/u', h)"), "typescript"),
        vec!["http_routing"],
        "app.get must resolve to http_routing (P7) not data_access (P9)"
    );
}

#[test]
fn axios_get_is_data_access_not_http_routing() {
    assert_eq!(
        classify_hint(&hint("axios.get(url)"), "typescript"),
        vec!["data_access"],
        "axios.get must stay in data_access — receiver axios is outside the http_routing allowlist"
    );
}

// ── TS/JS server route positives ──────────────────────────────────────────────

#[test]
fn router_post_is_http_routing() {
    assert_eq!(
        classify_hint(&hint("router.post('/user', cb)"), "typescript"),
        vec!["http_routing"]
    );
}

#[test]
fn fastify_route_is_http_routing() {
    assert_eq!(
        classify_hint(&hint("fastify.route({ method: 'GET' })"), "typescript"),
        vec!["http_routing"]
    );
}

#[test]
fn server_use_is_http_routing() {
    assert_eq!(
        classify_hint(&hint("server.use(middleware)"), "typescript"),
        vec!["http_routing"]
    );
}

#[test]
fn srv_all_is_http_routing() {
    assert_eq!(
        classify_hint(&hint("srv.all('*', h)"), "javascript"),
        vec!["http_routing"]
    );
}

#[test]
fn app_delete_is_http_routing() {
    assert_eq!(
        classify_hint(&hint("app.delete('/user/:id', h)"), "typescript"),
        vec!["http_routing"]
    );
}

// ── TS/JS client call negatives (stay data_access) ───────────────────────────

#[test]
fn client_get_is_data_access_not_http_routing() {
    // client is not in the http_routing receiver allowlist
    let result = classify_hint(&hint("client.get(url)"), "typescript");
    assert!(
        !result.contains(&"http_routing"),
        "client.get must not be http_routing; got {result:?}"
    );
}

#[test]
fn fetch_is_data_access_not_http_routing() {
    assert_eq!(
        classify_hint(&hint("fetch(\"/api/users\")"), "typescript"),
        vec!["data_access"]
    );
}

// ── Go route positives ────────────────────────────────────────────────────────

#[test]
fn go_http_handle_func_is_http_routing() {
    assert_eq!(
        classify_hint(&hint("http.HandleFunc(\"/\", h)"), "go"),
        vec!["http_routing"]
    );
}

#[test]
fn go_gin_r_get_is_http_routing() {
    assert_eq!(
        classify_hint(&hint("r.GET(\"/users\", h)"), "go"),
        vec!["http_routing"]
    );
}

#[test]
fn go_echo_e_post_is_http_routing() {
    assert_eq!(
        classify_hint(&hint("e.POST(\"/user\", h)"), "go"),
        vec!["http_routing"]
    );
}

#[test]
fn go_mux_handle_is_http_routing() {
    assert_eq!(
        classify_hint(&hint("mux.Handle(\"/\", h)"), "go"),
        vec!["http_routing"]
    );
}

// ── Python route positives ────────────────────────────────────────────────────

#[test]
fn python_add_url_rule_is_http_routing() {
    assert_eq!(
        classify_hint(&hint("app.add_url_rule('/u', view_func=h)"), "python"),
        vec!["http_routing"]
    );
}

// ── list_categories includes http_routing ────────────────────────────────────

#[test]
fn list_categories_includes_http_routing() {
    let catalog = sdivi_core::list_categories();
    let names: Vec<&str> = catalog.categories.iter().map(|c| c.name.as_str()).collect();
    assert!(
        names.contains(&"http_routing"),
        "list_categories must include 'http_routing'"
    );
}
