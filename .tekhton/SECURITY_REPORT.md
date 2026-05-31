## Summary
M36.1 is a purely additive pattern-category change: it registers `"decorator"` as a new tree-sitter node kind in the TS/JS extraction arrays and wires it into the static classification dispatch table (`category_for_node_kind`). No new code paths touch authentication, cryptography, network, file system I/O, or user-controlled input beyond what the existing extraction pipeline already handles. All new code is confined to compile-time string constants and a routing branch. Decorator node text is captured via the pre-existing `truncate_to_256_bytes` bound, consistent with all other node kinds. The change presents no exploitable surface.

## Findings
None

## Verdict
CLEAN
