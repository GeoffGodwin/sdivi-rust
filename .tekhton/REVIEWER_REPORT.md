# Reviewer Report
Review cycle: 1 of 2
Reviewer: Code Review Agent

---

## Verdict
APPROVED_WITH_NOTES

---

## Complex Blockers (senior coder)
None

---

## Simple Blockers (jr coder)
None

---

## Non-Blocking Notes
- `extract.rs:37` — `text.find("use ")` is correct for `use_declaration` nodes (tree-sitter guarantees no preceding string literals), but a brief comment on WHY `find` rather than `strip_prefix` would help future readers since the old fallback was a latent bug for years.
- `.github/workflows/verify-leiden.yml:48` — `cargo test --workspace --features verify-leiden -p sdi-detection` combines `--workspace` with `-p`; the recommended form when targeting one crate with a crate-local feature is `cargo test -p sdi-detection --features verify-leiden` to avoid ambiguous feature propagation to other workspace crates.
- Pre-existing LOW security findings (not introduced by this PR): TOCTOU in `crates/sdi-config/src/load.rs:98` and `boundary.rs:60`, terminal injection via TOML key in `load.rs:111` — flagged by the security agent; should be addressed in a dedicated cleanup pass.

---

## Coverage Gaps
- `crates/sdi-lang-rust/tests/extract_behavior.rs` — pre-existing: `collect_hints_long_unicode_text_truncated_at_char_boundary` fails because truncation can emit up to 257 bytes for a 2-byte char starting at byte 255 (off-by-one in the `take_while(*i < 256)` guard).
- `crates/sdi-lang-rust/tests/extract_behavior.rs` — pre-existing: `pub_fn_inside_pub_mod_not_in_top_level_exports` fails because `extract_exports` recurses unconditionally into `mod_item` children, collecting nested `pub fn` items as top-level exports.
- `crates/sdi-cli/tests/version.rs` — pre-existing: `version_flag_prints_crate_version` hardcodes `"0.0.1"` but `sdi-cli` is at `0.0.3`; either use `env!("CARGO_PKG_VERSION")` in the test or update the literal.

---

## Drift Observations
- `crates/sdi-lang-rust/src/extract.rs:103–113` — `collect_hints` truncation logic: `take_while(|(i, _)| *i < 256)` keeps chars whose START byte is < 256, so a 2-byte char at position 255 yields `end = 257`. This is the root cause of the test failure above; the fix is `take_while(|(i, _)| *i + c.len_utf8() <= 256)` (or equivalently, use `floor_char_boundary` once stabilised).
- `crates/sdi-lang-rust/src/extract.rs:62–75` — `extract_exports` pushes all children of every node including nested `mod_item` children, so `pub fn` items inside `pub mod` blocks are collected as if they were top-level exports. The intended behaviour (top-level only) would require stopping recursion at `mod_item` boundaries.
- `crates/sdi-cli/src/commands/init.rs:83–84` — the `config_path.exists()` → `load_with_paths` pattern inherits the TOCTOU race noted by the security agent (same pattern as `load.rs`); a symlink swap between the `.exists()` check and the subsequent read would silently validate the wrong file.
