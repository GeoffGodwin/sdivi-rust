## Summary
M33 replaces `category_for_node_kind` (node-kind-only) with `classify_hint` (node-kind + callee-text) in the `build_catalog` loop. The change touches only the pattern-classification path: no authentication, no cryptography, no network I/O, and no privilege-sensitive operations are involved. Source text extracted by tree-sitter is passed to Rust `regex`-crate matchers (linear-time DFA, immune to catastrophic backtracking). Fixture files extend test corpora with representative calls; none are executed at runtime. No new security-relevant code paths are introduced.

## Findings
None

## Verdict
CLEAN
