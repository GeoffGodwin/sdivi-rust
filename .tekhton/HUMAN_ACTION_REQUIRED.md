# Human Action Required

The pipeline identified items that need your attention. Review each item
and check it off when addressed. The pipeline will display a banner until
all items are resolved.

## Action Items
- [ ] [2026-04-28 | Source: architect] **`init.rs` stdout/stderr contract** — `crates/sdi-cli/src/commands/init.rs:84,98,103–105` writes progress lines (`"sdi: created .sdi/config.toml"`, `"sdi: detected languages: ..."`) to stdout via `println!`. Rule 8 (CLAUDE.md) reserves stdout for snapshot JSON, summaries, and table output and assigns logs/progress/warnings to stderr. The integration tests currently pin these on stdout, which means the choice is intentional but undocumented. A decision is needed before the 0.1.0 stdout/stderr contract is locked: either (a) carve out `sdi init` as a "setup command" whose output goes to stdout by deliberate exception, documenting that exception in CLAUDE.md Rule 8, or (b) move all `println!` in `init.rs` to `eprintln!` and update the integration tests to assert on stderr. The two choices produce a different observable contract for shell scripts and CI recipes.
- [ ] [2026-04-28 | Source: architect] **`serde_yaml` re-evaluation** — `Cargo.toml:41` notes `serde_yaml = "0.9"` is unmaintained; the note targets M10 for revisit. The original M01 drift observation requested revisit in M02 once the YAML loader was written. M02 is complete and the YAML loader (`crates/sdi-config/src/boundary.rs`) is live. The question is now concrete: accept the dependency risk through M10, or switch to a maintained fork (`serde_yml`) now while the YAML surface is small. The change to `serde_yml` is a one-line `Cargo.toml` swap plus a crate rename (`serde_yaml::` → `serde_yml::`) in `boundary.rs` and possibly `load.rs`. Decide the timeline. ---
