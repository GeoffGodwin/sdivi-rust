# Reviewer Report
**Date:** 2026-05-01
**Audit cycle:** post-M16 (non-blocking notes sweep, cycle 2)
**Reviewer:** Code Review Agent

---

## Verdict
APPROVED_WITH_NOTES

---

## Complex Blockers (senior coder)
- None

---

## Simple Blockers (jr coder)
- None

---

## Non-Blocking Notes
- Three security-agent findings in `crates/sdi-pipeline/src/commit_extract.rs` remain unfixed: MEDIUM (rev-parse without `--` separator at line 40-47), LOW (tar without `--no-absolute-filenames` at line 115-120), LOW (stderr verbatim in error variants at line 13-33). Coder dismissed all three as "handled by security pipeline / informational." All three were marked `fixable:yes` by the security agent and each requires a one-line code change. They should be applied in the next sweep rather than carried indefinitely as deferred non-blocking items.

---

## Coverage Gaps
- None

---

## Drift Observations
- `bindings/sdi-wasm/src/exports.rs:160-162` — `change_coupling: None` intentional gap is tracked only by a TODO comment inside the file. No corresponding ADL entry or issue exists to schedule the fix post-MVP. Risk of the TODO being silently forgotten.
- `bindings/sdi-wasm/src/types.rs:46-48` — `WasmLeidenConfigInput` missing `edge_weights` tracked as ADL-4. Verify ADL-4 actually exists in the architecture log; if not, create the entry so the gap is formally tracked.
- `.tekhton/NON_BLOCKING_LOG.md` — all 9 items are marked `[x]` (resolved) but items 3, 6, and 7 were deferred rather than fixed. The log offers no way to distinguish "resolved by fixing" from "resolved by deferring," which will obscure the true open count in future audits. Consider a `[deferred]` marker for clarity.
