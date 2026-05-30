//! Unit tests for `classify_hint` dispatch and per-language `matches_callee` functions.

use sdivi_patterns::queries::{
    async_patterns, classify_hint, data_access, logging, resource_management,
};
use sdivi_patterns::PatternHintInput;

fn hint(node_kind: &str, text: &str) -> PatternHintInput {
    PatternHintInput {
        node_kind: node_kind.to_string(),
        text: text.to_string(),
    }
}

// ── classify_hint dispatch ────────────────────────────────────────────────────

#[test]
fn classify_hint_console_log_is_logging() {
    assert_eq!(
        classify_hint(&hint("call_expression", "console.log(\"x\")"), "typescript"),
        vec!["logging"]
    );
}

#[test]
fn classify_hint_async_beats_logging() {
    // Priority: async_patterns > logging > data_access.
    assert_eq!(
        classify_hint(
            &hint("call_expression", "promise.then(resolve)"),
            "typescript"
        ),
        vec!["async_patterns"]
    );
}

#[test]
fn classify_hint_fetch_is_data_access_in_typescript() {
    assert_eq!(
        classify_hint(
            &hint("call_expression", "fetch(\"/api/users\")"),
            "typescript"
        ),
        vec!["data_access"]
    );
}

#[test]
fn classify_hint_unrecognised_call_returns_empty() {
    assert_eq!(
        classify_hint(&hint("call_expression", "Math.max(a, b)"), "typescript"),
        Vec::<&str>::new()
    );
    assert_eq!(
        classify_hint(&hint("call", "len(x)"), "python"),
        Vec::<&str>::new()
    );
}

#[test]
fn classify_hint_macro_invocation_logging_rust() {
    assert_eq!(
        classify_hint(&hint("macro_invocation", "tracing::info!(\"hi\")"), "rust"),
        vec!["logging"]
    );
    assert_eq!(
        classify_hint(&hint("macro_invocation", "println!(\"x\")"), "rust"),
        vec!["logging"]
    );
}

#[test]
fn classify_hint_macro_invocation_resource_management_rust() {
    assert_eq!(
        classify_hint(&hint("macro_invocation", "vec![1, 2, 3]"), "rust"),
        vec!["resource_management"]
    );
    assert_eq!(
        classify_hint(&hint("macro_invocation", "drop!(handle)"), "rust"),
        vec!["resource_management"]
    );
}

#[test]
fn classify_hint_falls_through_for_non_call_kinds() {
    assert_eq!(
        classify_hint(&hint("try_expression", "foo()?"), "rust"),
        vec!["error_handling"]
    );
    assert_eq!(
        classify_hint(&hint("await_expression", "future.await"), "rust"),
        vec!["async_patterns"]
    );
}

#[test]
fn classify_hint_unknown_kind_returns_empty() {
    assert_eq!(
        classify_hint(&hint("unknown_xyz", "whatever"), "rust"),
        Vec::<&str>::new()
    );
}

// Disjoint-regex invariant: for each test case, at most one matches_callee returns true.
#[test]
fn disjoint_regex_invariant_for_typescript_samples() {
    let samples = [
        ("promise.then(resolve)", "typescript"),
        ("console.log(\"x\")", "typescript"),
        ("fetch(\"/api\")", "typescript"),
        ("Math.max(a, b)", "typescript"),
    ];
    for (text, lang) in samples {
        let a = async_patterns::matches_callee(text, lang);
        let l = logging::matches_callee(text, lang);
        let d = data_access::matches_callee(text, lang);
        let matches = [a, l, d].iter().filter(|&&b| b).count();
        assert!(
            matches <= 1,
            "multiple categories matched for ({text}, {lang}): async={a} logging={l} data_access={d}"
        );
    }
}

// ── data_access::matches_callee ───────────────────────────────────────────────

#[test]
fn data_access_matches_fetch_typescript() {
    assert!(data_access::matches_callee(
        "fetch(\"/api/users\")",
        "typescript"
    ));
    assert!(data_access::matches_callee(
        "axios.get(\"/api\")",
        "typescript"
    ));
    assert!(!data_access::matches_callee("Math.max(a, b)", "typescript"));
}

#[test]
fn data_access_matches_db_call_go() {
    assert!(data_access::matches_callee("db.query(sql)", "go"));
    assert!(data_access::matches_callee(
        "sql.Open(\"postgres\", dsn)",
        "go"
    ));
    assert!(!data_access::matches_callee("fmt.Println(\"x\")", "go"));
}

#[test]
fn data_access_matches_python() {
    assert!(data_access::matches_callee("cursor.execute(sql)", "python"));
    assert!(data_access::matches_callee("requests.get(url)", "python"));
    assert!(data_access::matches_callee("open(path, \"r\")", "python"));
    assert!(!data_access::matches_callee("len(x)", "python"));
    assert!(!data_access::matches_callee("print(x)", "python"));
}

#[test]
fn data_access_rust_returns_false() {
    // Rust data-access is library-shaped and deferred to a future pass.
    assert!(!data_access::matches_callee("sqlx::query!(sql)", "rust"));
    assert!(!data_access::matches_callee("reqwest::get(url)", "rust"));
}

// ── logging::matches_callee ───────────────────────────────────────────────────

#[test]
fn logging_matches_typescript() {
    assert!(logging::matches_callee("console.log(\"x\")", "typescript"));
    assert!(logging::matches_callee("logger.info(\"x\")", "typescript"));
    assert!(logging::matches_callee("log.debug(\"x\")", "typescript"));
    assert!(!logging::matches_callee("Math.max(a, b)", "typescript"));
    assert!(!logging::matches_callee(
        "myconsole.log(\"x\")",
        "typescript"
    ));
}

#[test]
fn logging_matches_python() {
    assert!(logging::matches_callee("logging.info(\"x\")", "python"));
    assert!(logging::matches_callee("print(x)", "python"));
    assert!(!logging::matches_callee("printer(x)", "python"));
    assert!(!logging::matches_callee("len(x)", "python"));
}

#[test]
fn logging_matches_go() {
    assert!(logging::matches_callee("fmt.Println(\"x\")", "go"));
    assert!(logging::matches_callee("fmt.Printf(\"%v\", x)", "go"));
    assert!(!logging::matches_callee("myfmt.Println(\"x\")", "go"));
    assert!(!logging::matches_callee("db.query(sql)", "go"));
}

#[test]
fn logging_matches_rust() {
    assert!(logging::matches_callee("tracing::info!(\"hi\")", "rust"));
    assert!(logging::matches_callee("log::debug!(\"x\")", "rust"));
    assert!(logging::matches_callee("println!(\"x\")", "rust"));
    assert!(logging::matches_callee("eprintln!(\"err\")", "rust"));
    assert!(logging::matches_callee("dbg!(val)", "rust"));
    assert!(!logging::matches_callee("vec![1, 2]", "rust"));
    assert!(!logging::matches_callee("assert!(cond)", "rust"));
}

#[test]
fn logging_matches_java() {
    assert!(logging::matches_callee("System.out.println(\"x\")", "java"));
    assert!(logging::matches_callee("logger.info(\"x\")", "java"));
    assert!(logging::matches_callee("LOG.debug(\"x\")", "java"));
    assert!(!logging::matches_callee("MyClass.method()", "java"));
}

// ── async_patterns::matches_callee ───────────────────────────────────────────

#[test]
fn async_patterns_matches_promise_chain_typescript() {
    assert!(async_patterns::matches_callee(
        "promise.then(resolve)",
        "typescript"
    ));
    assert!(async_patterns::matches_callee(
        "fetch(url).catch(err => {})",
        "javascript"
    ));
    assert!(async_patterns::matches_callee(
        "p.finally(() => {})",
        "typescript"
    ));
    assert!(!async_patterns::matches_callee(
        "Math.max(a, b)",
        "typescript"
    ));
    assert!(!async_patterns::matches_callee("getNext()", "typescript"));
}

#[test]
fn async_patterns_other_languages_return_false() {
    assert!(!async_patterns::matches_callee("promise.then(x)", "rust"));
    assert!(!async_patterns::matches_callee("promise.then(x)", "python"));
}

// ── resource_management::excludes_callee ─────────────────────────────────────

#[test]
fn resource_management_excludes_rust_logging_macros() {
    assert!(resource_management::excludes_callee(
        "tracing::info!(\"hi\")",
        "rust"
    ));
    assert!(resource_management::excludes_callee(
        "println!(\"x\")",
        "rust"
    ));
    assert!(resource_management::excludes_callee(
        "eprintln!(\"err\")",
        "rust"
    ));
    assert!(!resource_management::excludes_callee(
        "vec![1, 2, 3]",
        "rust"
    ));
    assert!(!resource_management::excludes_callee(
        "drop!(handle)",
        "rust"
    ));
}

#[test]
fn resource_management_other_languages_always_false() {
    assert!(!resource_management::excludes_callee(
        "println!(\"x\")",
        "typescript"
    ));
    assert!(!resource_management::excludes_callee(
        "tracing::info!(\"x\")",
        "java"
    ));
}

// ── Disjoint-regex invariant: additional language coverage ────────────────────
//
// The existing `disjoint_regex_invariant_for_typescript_samples` covers TS only.
// The milestone Watch For requires the invariant be verified across all languages.
// Each test below checks that for a given (text, language) sample pair, at most
// one of the three call-path classifiers (async_patterns, logging, data_access)
// returns true — the regex tables must remain disjoint per language.

#[test]
fn disjoint_regex_invariant_for_python_samples() {
    let samples = [
        ("logging.info(\"x\")", "python"),
        ("cursor.execute(sql)", "python"),
        ("print(x)", "python"),
        ("requests.get(url)", "python"),
        ("open(path, \"r\")", "python"),
        ("httpx.get(url)", "python"),
        ("session.commit()", "python"),
        ("conn.cursor()", "python"),
        ("len(x)", "python"),
        ("int(val)", "python"),
    ];
    for (text, lang) in samples {
        let a = async_patterns::matches_callee(text, lang);
        let l = logging::matches_callee(text, lang);
        let d = data_access::matches_callee(text, lang);
        let count = [a, l, d].iter().filter(|&&b| b).count();
        assert!(
            count <= 1,
            "disjoint-regex invariant violated for ({text:?}, {lang:?}): \
             async={a} logging={l} data_access={d} — at most one must match"
        );
    }
}

#[test]
fn disjoint_regex_invariant_for_go_samples() {
    let samples = [
        ("fmt.Println(\"x\")", "go"),
        ("fmt.Printf(\"%v\", x)", "go"),
        ("fmt.Errorf(\"msg\")", "go"),
        ("fmt.Fprintf(w, \"x\")", "go"),
        ("db.query(sql)", "go"),
        ("db.Query(ctx, sql)", "go"),
        ("sql.Open(\"postgres\", dsn)", "go"),
        ("fetch(\"/api\")", "go"),
        ("write(data)", "go"),
        ("read(buf)", "go"),
        ("os.Exit(1)", "go"),
        ("len(slice)", "go"),
    ];
    for (text, lang) in samples {
        let a = async_patterns::matches_callee(text, lang);
        let l = logging::matches_callee(text, lang);
        let d = data_access::matches_callee(text, lang);
        let count = [a, l, d].iter().filter(|&&b| b).count();
        assert!(
            count <= 1,
            "disjoint-regex invariant violated for ({text:?}, {lang:?}): \
             async={a} logging={l} data_access={d} — at most one must match"
        );
    }
}

/// Disjoint-regex invariant for Rust call_expression context.
///
/// Note: in practice Rust uses macro_invocation (not call_expression) for
/// tracing::info! etc. This test verifies the invariant still holds for Rust
/// even when those texts appear in a call_expression dispatch context.
#[test]
fn disjoint_regex_invariant_for_rust_call_samples() {
    let samples = [
        ("tracing::info!(\"x\")", "rust"),
        ("log::debug!(\"x\")", "rust"),
        ("println!(\"x\")", "rust"),
        ("eprintln!(\"err\")", "rust"),
        ("dbg!(val)", "rust"),
        ("sqlx::query!(sql)", "rust"),
        ("reqwest::get(url)", "rust"),
        ("some_function()", "rust"),
        ("vec![1, 2]", "rust"),
    ];
    for (text, lang) in samples {
        let a = async_patterns::matches_callee(text, lang);
        let l = logging::matches_callee(text, lang);
        let d = data_access::matches_callee(text, lang);
        let count = [a, l, d].iter().filter(|&&b| b).count();
        assert!(
            count <= 1,
            "disjoint-regex invariant violated for ({text:?}, {lang:?}): \
             async={a} logging={l} data_access={d} — at most one must match"
        );
    }
}

/// For Rust macro_invocation dispatch, `resource_management::excludes_callee` and
/// `logging::matches_callee` must use identical regex semantics: whenever
/// `excludes_callee` returns true, `logging::matches_callee` must also return true.
/// The shared regex is the invariant that makes the double-guard dispatch correct.
#[test]
fn disjoint_regex_invariant_for_rust_macro_invocations() {
    let samples = [
        ("tracing::info!(\"x\")", "rust"),
        ("tracing::warn!(\"w\")", "rust"),
        ("log::debug!(\"x\")", "rust"),
        ("log::error!(\"e\")", "rust"),
        ("println!(\"x\")", "rust"),
        ("eprintln!(\"err\")", "rust"),
        ("print!(\"p\")", "rust"),
        ("eprint!(\"e\")", "rust"),
        ("dbg!(val)", "rust"),
        // These must NOT be excluded — they stay as resource_management.
        ("vec![1, 2, 3]", "rust"),
        ("drop!(handle)", "rust"),
        ("assert!(cond)", "rust"),
        ("format!(\"x\")", "rust"),
    ];
    for (text, lang) in samples {
        let excludes = resource_management::excludes_callee(text, lang);
        let is_logging = logging::matches_callee(text, lang);
        if excludes {
            assert!(
                is_logging,
                "resource_management::excludes_callee({text:?}, {lang:?}) is true but \
                 logging::matches_callee is false — they use the same regex, so both \
                 must agree; a divergence means one regex was updated without the other"
            );
        }
        // Also verify: if logging::matches_callee is true for a macro, excludes_callee
        // must also be true (symmetric agreement for Rust macros).
        if is_logging && lang == "rust" {
            assert!(
                excludes,
                "logging::matches_callee({text:?}, {lang:?}) is true but \
                 resource_management::excludes_callee is false — symmetric agreement \
                 failure; update RUST_LOGGING_RE in resource_management to match logging::RUST_RE"
            );
        }
    }
}
