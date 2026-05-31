## Summary

M40 adds the `collection_pipelines` pattern category: a single new Rust module (`collection_pipelines.rs`) containing one `LazyLock<Regex>` static and a pure function `matches_callee(&str, &str) -> bool`, wired into `CALL_DISPATCH` at slot P10 and the `categories.rs` contract table. The change involves no I/O, no network calls, no authentication logic, no cryptography, and no user-controlled data beyond callee-text strings already extracted by tree-sitter (a trusted, in-process component). There are no security findings.

## Findings

None

## Verdict

CLEAN

---

*Reviewer notes (not findings):*

`TS_JS_RE` was evaluated for ReDoS potential. The regex `\.(map|filter|reduce|flatMap|forEach|find|findIndex|some|every|flat)\(` is a flat alternation of fixed-length literals bounded by literal `.` and `(` anchors. No nested quantifiers, no ambiguous paths, no backtracking traps. The `regex` crate's NFA/DFA engine runs in O(n) time on input length regardless. The `language` match arm is a closed set (`"typescript" | "javascript" | "go" | "java"`) returning `false` for all other inputs, providing an additional fail-fast gate before regex evaluation.
