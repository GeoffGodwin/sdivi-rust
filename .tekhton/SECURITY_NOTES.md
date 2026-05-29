# Security Notes

Generated: 2026-05-29 14:07:23

## Non-Blocking Findings (MEDIUM/LOW)
- [LOW] [category:A03] [bindings/sdivi-wasm/src/exports.rs:186] fixable:yes — The `classify_hint` WASM export accepts a caller-supplied `hint.text` field with no enforcement of the documented 256-byte truncation. The `PatternHintInput` docstring states "truncated to 256 bytes upstream (per the PatternHint contract)" but the WASM API is the upstream for JS callers — there is no upstream enforcer. A malicious caller can pass an arbitrarily long string. All current regex patterns carry O(n) guarantees from the `regex` crate (no backreferences, no catastrophic backtracking), so this is proportional overhead only, not a ReDoS vulnerability. Fix: enforce the cap at the WASM boundary — truncate `hint.text` to 256 bytes (nearest char boundary) in `exports.rs:classify_hint` before constructing `PatternHintInput`.
