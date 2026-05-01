# Security Notes

Generated: 2026-05-01 09:21:18

## Non-Blocking Findings (MEDIUM/LOW)
- [LOW] [category:A04] [crates/sdi-core/src/compute/thresholds.rs:132] fixable:yes — `boundary_violation_delta` (i64) is cast to f64 via `as f64` before threshold comparison. For violation counts above 2^53 the cast loses precision, which could cause a breach to go undetected. Pre-existing; not introduced by this PR. Practical violation counts will never approach that range, but a `min(delta, i64::MAX_SAFE_F64) as f64` clamp or explicit `TryFrom` would be more correct.
