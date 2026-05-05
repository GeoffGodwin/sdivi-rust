# Security Notes

Generated: 2026-05-04 22:59:11

## Non-Blocking Findings (MEDIUM/LOW)
- [LOW] [category:A03] [bindings/sdivi-wasm/src/weight_keys.rs:25-34] fixable:yes — `parse_wasm_edge_weights` rejects `NaN` and negative values but does not call `weight.is_infinite()`. `f64::INFINITY` passes both guards and propagates into the Leiden algorithm. The documented contract in the function-level rustdoc and in `types.rs:55` explicitly states "finite", so this is an implementation gap. Fix: add `if weight.is_infinite() { return Err(format!("edge weight for key \"{key}\" is infinite; all weights must be finite")); }` after the NaN check (line 25). A matching unit test `rejects_infinite_weight` should be added to the `#[cfg(test)] mod tests` block.
