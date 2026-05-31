//! Tests documenting the scope boundaries of the `http_routing` category.
//!
//! Two roles:
//! 1. Assert that patterns intentionally NOT detected remain undetected —
//!    regression guards against accidentally widening the matching surface.
//! 2. Document the limitations so future contributors understand where the
//!    v0 callee-text model stops.
//!
//! ## Next.js App Router limitation (reviewer coverage gap)
//!
//! Next.js App Router route handlers use function-export syntax
//! (`export async function GET(request: Request) {}`), not `call_expression`
//! nodes. There is no receiver to match against the allowlist; the v0
//! callee-text model cannot detect these. They remain undetected in v0.
//!
//! ## Idiosyncratic receiver names
//!
//! The receiver allowlist is finite: `app`, `router`, `fastify`, `server`,
//! `srv`. A server variable named outside the allowlist (`api`, `hono`,
//! `myRouter`) will not be classified as `http_routing`. This is the documented
//! precision tradeoff described in `http_routing.rs`.

use sdivi_patterns::queries::{classify_hint, http_routing};
use sdivi_patterns::PatternHintInput;

fn call_expr(text: &str) -> PatternHintInput {
    PatternHintInput {
        node_kind: "call_expression".to_string(),
        text: text.to_string(),
    }
}

// ── Next.js App Router limitation ─────────────────────────────────────────────
// App Router route handlers are function declarations, not call_expression nodes.
// If a bare `GET(request)` call_expression appears (invoking the exported handler),
// it has no receiver — the allowlist never fires.

#[test]
fn nextjs_app_router_bare_get_is_not_http_routing() {
    assert!(
        !http_routing::matches_callee("GET(request)", "typescript"),
        "bare GET() without a receiver must not match http_routing"
    );
}

#[test]
fn nextjs_app_router_bare_post_is_not_http_routing() {
    assert!(
        !http_routing::matches_callee("POST(request)", "typescript"),
        "bare POST() without a receiver must not match http_routing"
    );
}

#[test]
fn nextjs_app_router_bare_put_is_not_http_routing() {
    assert!(!http_routing::matches_callee("PUT(request)", "typescript"));
}

#[test]
fn nextjs_app_router_bare_delete_is_not_http_routing() {
    assert!(!http_routing::matches_callee("DELETE(request)", "typescript"));
}

#[test]
fn nextjs_app_router_bare_patch_is_not_http_routing() {
    assert!(!http_routing::matches_callee("PATCH(request)", "typescript"));
}

#[test]
fn classify_hint_bare_get_is_not_http_routing() {
    // Verify via the full CALL_DISPATCH path — bare `GET(` must not resolve to http_routing
    let result = classify_hint(&call_expr("GET(request)"), "typescript");
    assert!(
        !result.contains(&"http_routing"),
        "classify_hint must not return http_routing for bare GET(); got {result:?}"
    );
}

// ── Idiosyncratic receiver limitation ─────────────────────────────────────────
// Variables named outside the allowlist (`api`, `hono`, `myRouter`, `app2`) are
// not detected. Receiver-type inference would require a type-info pass — out of
// scope for the v0 callee-text model.

#[test]
fn idiosyncratic_api_receiver_is_not_http_routing() {
    // `const api = express(); api.get(...)` — `api` is not in the TS/JS allowlist
    assert!(
        !http_routing::matches_callee("api.get('/path', h)", "typescript"),
        "receiver `api` is not in the allowlist"
    );
}

#[test]
fn idiosyncratic_hono_variable_is_not_http_routing() {
    // Hono app named `hono` rather than `app` / `server` / etc.
    assert!(!http_routing::matches_callee("hono.get('/path', h)", "typescript"));
}

#[test]
fn idiosyncratic_my_router_variable_is_not_http_routing() {
    assert!(!http_routing::matches_callee("myRouter.get('/path', h)", "typescript"));
}

#[test]
fn idiosyncratic_app2_variable_is_not_http_routing() {
    assert!(!http_routing::matches_callee("app2.get('/path', h)", "typescript"));
}

#[test]
fn classify_hint_idiosyncratic_receiver_falls_through_to_data_access() {
    // `api.get(...)` — not in http_routing allowlist, falls through CALL_DISPATCH
    // to data_access (P9) because `.get(` matches the data-access verb regex
    let result = classify_hint(&call_expr("api.get('/path', h)"), "typescript");
    assert!(
        !result.contains(&"http_routing"),
        "idiosyncratic receiver `api` must not resolve to http_routing; got {result:?}"
    );
}

// ── Go receivers — full allowlist integration coverage ────────────────────────
// The http_routing.rs unit tests cover these via matches_callee directly.
// These tests exercise the full classify_hint dispatch path for every Go
// receiver in the spec that was not already in category_contract_m41.rs.

#[test]
fn go_engine_group_via_classify_hint() {
    assert_eq!(
        classify_hint(&call_expr("engine.Group(\"/api\")"), "go"),
        vec!["http_routing"]
    );
}

#[test]
fn go_g_get_via_classify_hint() {
    assert_eq!(
        classify_hint(&call_expr("g.GET(\"/users\", h)"), "go"),
        vec!["http_routing"]
    );
}

#[test]
fn go_rg_post_via_classify_hint() {
    assert_eq!(
        classify_hint(&call_expr("rg.POST(\"/user\", h)"), "go"),
        vec!["http_routing"]
    );
}

#[test]
fn go_router_delete_via_classify_hint() {
    // Go `router` receiver (distinct from TS/JS `router`)
    assert_eq!(
        classify_hint(&call_expr("router.DELETE(\"/user/:id\", h)"), "go"),
        vec!["http_routing"]
    );
}

#[test]
fn go_mux_any_via_classify_hint() {
    assert_eq!(
        classify_hint(&call_expr("mux.Any(\"/\", h)"), "go"),
        vec!["http_routing"]
    );
}

#[test]
fn go_e_patch_via_classify_hint() {
    assert_eq!(
        classify_hint(&call_expr("e.PATCH(\"/item/:id\", h)"), "go"),
        vec!["http_routing"]
    );
}

// ── Python — receiver-agnostic coverage ──────────────────────────────────────
// The Python regex matches `.add_url_rule(` on any receiver (Flask app or
// Blueprint). Verify that Blueprint receivers also classify correctly.

#[test]
fn python_blueprint_add_url_rule_via_classify_hint() {
    assert_eq!(
        classify_hint(&call_expr("bp.add_url_rule('/users', view_func=h)"), "python"),
        vec!["http_routing"]
    );
}

#[test]
fn python_any_receiver_add_url_rule_matches_callee() {
    // Receiver-agnostic: any object calling `.add_url_rule(` matches
    assert!(http_routing::matches_callee("x.add_url_rule('/p', view_func=h)", "python"));
}
