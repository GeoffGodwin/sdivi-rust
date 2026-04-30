## Summary
This changeset consists of four comment-only edits and one substantive code change: the `ThresholdsInput::default().today` sentinel was corrected from `2026-01-01` (a past date) to `9999-12-31` (a far-future date). The sentinel change is a security improvement — it makes the default fail closed (all per-category overrides appear expired unless the caller supplies the real date), eliminating a footgun where the old past sentinel silently activated overrides that should have been evaluated against the real calendar date. The two previously reported LOW findings (TOCTOU in `load_toml_file` and terminal injection via TOML key names) were already resolved before this PR, as confirmed by the coder's Note 3/Note 6 verification. No authentication, cryptography, network, or user-input-handling code was touched. No new attack surface was introduced.

## Findings

- [LOW] [category:A04] [crates/sdi-core/src/compute/thresholds.rs:132] fixable:yes — `boundary_violation_delta` (i64) is cast to f64 via `as f64` before threshold comparison. For violation counts above 2^53 the cast loses precision, which could cause a breach to go undetected. Pre-existing; not introduced by this PR. Practical violation counts will never approach that range, but a `min(delta, i64::MAX_SAFE_F64) as f64` clamp or explicit `TryFrom` would be more correct.

## Verdict
CLEAN
