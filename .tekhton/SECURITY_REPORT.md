## Summary
M21 adds a WASM-facing input validation layer (`weight_keys.rs`) that converts
colon-separated edge-weight keys to native NUL-separated keys and rejects NaN,
negative, and malformed entries before passing weights to `sdivi_core::detect_boundaries`.
The change is scoped entirely to the `bindings/sdivi-wasm` crate; no auth, network,
cryptography, or filesystem paths are involved. One validation gap exists between the
documented contract ("finite") and the implemented checks (NaN + negative only):
`f64::INFINITY` silently passes through, which can corrupt Leiden intermediate
arithmetic and produce a nonsense analysis result rather than a clean error.

## Findings
- [LOW] [category:A03] [bindings/sdivi-wasm/src/weight_keys.rs:25-34] fixable:yes — `parse_wasm_edge_weights` rejects `NaN` and negative values but does not call `weight.is_infinite()`. `f64::INFINITY` passes both guards and propagates into the Leiden algorithm. The documented contract in the function-level rustdoc and in `types.rs:55` explicitly states "finite", so this is an implementation gap. Fix: add `if weight.is_infinite() { return Err(format!("edge weight for key \"{key}\" is infinite; all weights must be finite")); }` after the NaN check (line 25). A matching unit test `rejects_infinite_weight` should be added to the `#[cfg(test)] mod tests` block.

## Verdict
FINDINGS_PRESENT
