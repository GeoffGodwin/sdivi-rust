//! Node-kind and callee-text classification for concurrency patterns.
//!
//! ## Node-kind detection (Go only)
//!
//! - `go_statement` — goroutine launch (`go worker(ch)`)
//! - `select_statement` — channel multiplexing (`select { case msg := <-ch: ... }`)
//!
//! These are already collected by the Go adapter (`sdivi-lang-go`) and were
//! previously unclassified. No parsing change required.
//!
//! ## Callee-text detection
//!
//! - **TypeScript / JavaScript:** `Promise.all(…)`, `Promise.allSettled(…)`,
//!   `Promise.race(…)`, `Promise.any(…)` — registered at CALL_DISPATCH slot P11.
//! - **Python:** `asyncio.gather(…)`, `asyncio.create_task(…)`, `asyncio.wait(…)`,
//!   `asyncio.as_completed(…)`, `asyncio.run(…)`.
//!
//! ## CALL_DISPATCH slot
//!
//! Registered at P11 — lowest precedence, after `collection_pipelines` (P10).
//! `Promise.all(…)` does not overlap with `async_patterns` unless chained:
//! `Promise.all([]).then(cb)` as a single AST node matches both `async_patterns`
//! (P1, `.then(`) and `concurrency` (P11); P1 wins by precedence. The bare inner
//! `Promise.all([…])` call resolves to `concurrency`.
//!
//! ## Boundary with `async_patterns`
//!
//! `async_patterns` covers single-future `.await` and Promise-chain methods
//! (`.then`, `.catch`, `.finally`). `concurrency` covers multi-future coordination
//! (`Promise.all`) and goroutine/channel primitives — the two buckets are disjoint
//! in practice.
//!
//! ## Deferred
//!
//! - `defer_statement` belongs to `resource_management` (M45.1), not here.
//! - `tokio::spawn` / `thread::spawn` require adding `call_expression` to the Rust
//!   adapter's `PATTERN_KINDS` — a separate change with its own migration impact.
//! - Go channel operators (`ch <- x`, `<-ch`) and `sync.WaitGroup`/`Mutex` require
//!   operator/receiver extraction beyond the v0 node-kind model.

use std::sync::LazyLock;

use regex::Regex;

/// Tree-sitter node kinds that map to the `concurrency` category.
///
/// Emitted by the Go adapter (`sdivi-lang-go`). Classification for these node
/// kinds happens in `category_for_node_kind`, not in `CALL_DISPATCH`.
pub const NODE_KINDS: &[&str] = &["go_statement", "select_statement"];

// TypeScript / JavaScript — Promise multi-future coordination.
// Anchored at `^Promise\.` so bare `.then/catch/finally` chains stay in async_patterns.
static TS_JS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^Promise\.(all|allSettled|race|any)\(").expect("concurrency TS/JS regex is valid")
});

// Python — asyncio top-level coordination functions.
static PYTHON_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^asyncio\.(gather|create_task|wait|as_completed|run)\(")
        .expect("concurrency Python regex is valid")
});

/// Return `true` when `text` looks like a concurrent-execution coordination call.
///
/// Covers `Promise.all/allSettled/race/any` (TS/JS) and `asyncio.gather/
/// create_task/wait/as_completed/run` (Python). Go concurrency is detected via
/// node kind (`go_statement`, `select_statement`) rather than callee text.
///
/// # Examples
///
/// ```rust
/// use sdivi_patterns::queries::concurrency::matches_callee;
///
/// assert!(matches_callee("Promise.all([a, b])", "typescript"));
/// assert!(matches_callee("Promise.allSettled([p1, p2])", "javascript"));
/// assert!(matches_callee("Promise.race([a, b])", "javascript"));
/// assert!(matches_callee("Promise.any([a, b])", "typescript"));
/// assert!(matches_callee("asyncio.gather(*tasks)", "python"));
/// assert!(matches_callee("asyncio.create_task(coro())", "python"));
/// assert!(matches_callee("asyncio.run(main())", "python"));
/// assert!(!matches_callee("promise.then(r)", "typescript"));
/// assert!(!matches_callee("Promise.resolve(x)", "javascript"));
/// assert!(!matches_callee("asyncio.sleep(1)", "python"));
/// ```
pub fn matches_callee(text: &str, language: &str) -> bool {
    match language {
        "typescript" | "javascript" => TS_JS_RE.is_match(text),
        "python" => PYTHON_RE.is_match(text),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promise_all_matches_ts() {
        assert!(matches_callee("Promise.all([a, b])", "typescript"));
    }

    #[test]
    fn promise_all_settled_matches_js() {
        assert!(matches_callee("Promise.allSettled([p1, p2])", "javascript"));
    }

    #[test]
    fn promise_race_matches_js() {
        assert!(matches_callee("Promise.race([a, b])", "javascript"));
    }

    #[test]
    fn promise_any_matches_ts() {
        assert!(matches_callee("Promise.any([a, b])", "typescript"));
    }

    #[test]
    fn asyncio_gather_matches_python() {
        assert!(matches_callee("asyncio.gather(*tasks)", "python"));
    }

    #[test]
    fn asyncio_create_task_matches_python() {
        assert!(matches_callee("asyncio.create_task(coro())", "python"));
    }

    #[test]
    fn asyncio_wait_matches_python() {
        assert!(matches_callee("asyncio.wait(tasks)", "python"));
    }

    #[test]
    fn asyncio_as_completed_matches_python() {
        assert!(matches_callee("asyncio.as_completed(tasks)", "python"));
    }

    #[test]
    fn asyncio_run_matches_python() {
        assert!(matches_callee("asyncio.run(main())", "python"));
    }

    #[test]
    fn promise_then_does_not_match_ts() {
        assert!(!matches_callee("promise.then(r)", "typescript"));
    }

    #[test]
    fn promise_resolve_does_not_match_js() {
        assert!(!matches_callee("Promise.resolve(x)", "javascript"));
    }

    #[test]
    fn asyncio_sleep_does_not_match_python() {
        assert!(!matches_callee("asyncio.sleep(1)", "python"));
    }

    #[test]
    fn go_returns_false() {
        assert!(!matches_callee("go worker(ch)", "go"));
    }

    #[test]
    fn rust_returns_false() {
        assert!(!matches_callee("tokio::spawn(async {})", "rust"));
    }

    #[test]
    fn node_kinds_contains_go_statement() {
        assert!(NODE_KINDS.contains(&"go_statement"));
    }

    #[test]
    fn node_kinds_contains_select_statement() {
        assert!(NODE_KINDS.contains(&"select_statement"));
    }
}
