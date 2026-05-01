# Reviewer Report — M13 (Release Pipeline and Distribution)
## Review cycle: 2 of 4

## Verdict
APPROVED_WITH_NOTES

## Complex Blockers (senior coder)
None

## Simple Blockers (jr coder)
None

## Non-Blocking Notes

- [release.yml:230] `publish-npm` still uses `dtolnay/rust-toolchain@master` — a mutable branch ref — while all other jobs use `dtolnay/rust-toolchain@stable`. Should be `@stable` for consistency (security agent flagged LOW; persisted from prior cycle).
- [release.yml:159-160] `CARGO_REGISTRY_TOKEN` is still declared at the `publish-crates` job level, making it visible to `actions/checkout@v4` and `Swatinem/rust-cache@v2`. Scope it to only the `cargo publish` steps via step-level `env:` (security agent flagged LOW; persisted from prior cycle).
- [release.yml:132] `create-release` still includes `uses: actions/checkout@v4` but no subsequent step reads a file from the repo. The checkout is unnecessary; remove it to reduce attack surface (persisted from prior cycle).
- [release.yml] All third-party actions remain pinned to version tags (`@v4`, `@v2`, `@stable`) rather than immutable SHA digests. Pin to commit SHAs before this workflow runs against production credentials (security agent flagged MEDIUM; persisted from prior cycle).
- [crates/sdi-rust/Cargo.toml] `sdi-core = { workspace = true }` is declared as a direct dependency but `src/main.rs` only calls `sdi_cli::run()` — `sdi-core` is already a transitive dep via `sdi-cli`. The redundant direct dep adds noise (persisted from prior cycle).

## Coverage Gaps

- No dry-run validation of crates.io publish order in CI; a `cargo publish --dry-run -p sdi-lang-rust` step (after a dry-run of `sdi-parsing`) in `ci.yml` would catch dependency-ordering regressions early.
- `publish-npm` dry-run for RC tags does not verify that `.d.ts` and `.wasm` files are present in `pkg/` before the dry-run; a `ls -lh pkg/` step before `npm publish --dry-run` would surface build failures explicitly.

## ACP Verdicts

- ACP: `sdi-cli` exposed as library target to enable `cargo install sdi-rust` — **ACCEPT** (confirmed from cycle 1; no rework changed this decision).

## Drift Observations

- [release.yml] The prior-cycle drift observation about the comment block (lines 13–16) showing wrong publish order has been **resolved** — the comment now accurately reflects the corrected order.
- [CHANGELOG.md] Entries for `[0.0.1]`–`[0.0.16]` still contain internal development tracking noise (e.g. "Managed the human_action_required", "[MILESTONE 10 ✓]"). The coder noted this as out of scope. Flagged again so the user can clean it up before the public `0.1.0` tag.

## Prior Blocker Verification

- **FIXED** — `publish-crates` publish order was the single prior blocker. `release.yml` lines 166–187 now publish `sdi-config` → `sdi-parsing` (sleep 30) → `sdi-lang-*` → `sdi-graph` → ... The comment block at lines 13–15 was also corrected to match. No regressions introduced by the rework.
