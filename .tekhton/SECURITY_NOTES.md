# Security Notes

Generated: 2026-04-29 08:21:04

## Non-Blocking Findings (MEDIUM/LOW)
- [LOW] [category:A07] [crates/sdi-config/src/load.rs:98, crates/sdi-config/src/boundary.rs:60] fixable:yes — TOCTOU: both `load_toml_file` and `BoundarySpec::load` call `path.exists()` and then `fs::read_to_string` as separate operations. A symlink swap between the two syscalls could redirect the read to an unintended file. The idiomatic fix eliminates the race: call `fs::read_to_string` directly and match `Err(e) if e.kind() == ErrorKind::NotFound` to return `Ok(None)`.
- [LOW] [category:A03] [crates/sdi-config/src/load.rs:111] fixable:yes — Terminal injection: TOML quoted keys permit arbitrary Unicode (including ANSI escape sequences). The key name is formatted verbatim into an `eprintln!` warning. A malicious `.sdi/config.toml` in an untrusted repository could embed escape sequences (e.g. cursor-movement codes) that manipulate the developer's terminal when `sdi` is run. Fix: strip or sanitically escape the key before printing, e.g. limit to ASCII printable characters or use a `{key:?}` debug format to make embedded escapes visible as `\u{...}`.
- [LOW] [category:A06] [Cargo.toml:41] fixable:no — `serde_yaml = "0.9"` is unmaintained upstream (already acknowledged in the inline comment "revisit in M10"). No known CVEs exist at review time, but the lack of active maintenance means future vulnerabilities would not receive upstream patches. Tracked for M10; no action required before that milestone.
