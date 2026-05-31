## Summary

M41 adds the `http_routing` pattern category: a new Rust module (`http_routing.rs`) containing three `LazyLock<Regex>` statics and a pure function `matches_callee(&str, &str) -> bool`, wired into `CALL_DISPATCH` at slot P7 and the `categories.rs` contract table. The change involves no I/O, no network calls, no authentication logic, no cryptography, and no user-controlled data beyond callee-text strings already extracted by tree-sitter (a trusted, in-process component). All three regexes are anchored or use fixed-literal matching — no nested quantifiers, no ReDoS risk. There are no security findings.

## Findings

None

## Verdict

CLEAN

---

*Reviewer notes (not findings):*

All three regexes were evaluated for ReDoS potential:
- `TS_JS_RE`: `^(app|router|fastify|server|srv)\.(get|post|put|...) \(` — `^`-anchored, flat alternations of fixed-length literals, no backtracking traps.
- `GO_RE`: `^(http|mux|r|e|router|engine|g|rg)\.(HandleFunc|Handle|GET|...)` — identical structure, `^`-anchored, safe.
- `PYTHON_RE`: `\.add_url_rule\(` — pure literal substring match, O(n) in all cases.

The `regex` crate's NFA/DFA engine runs in O(n) time regardless. The `language` match arm in `matches_callee` is a closed set (`"typescript" | "javascript" | "go" | "python"`) returning `false` for all other inputs before regex evaluation.
