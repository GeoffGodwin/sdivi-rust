#### Milestone 4: Remaining Language Adapters (Python, TS, JS, Go, Java)

**Scope:** Implement `LanguageAdapter` for the five remaining default languages. Each in its own crate behind a Cargo feature flag. Compile-time grammar linking. Per-language test fixture.

**Deliverables:**
- `sdi-lang-python`, `sdi-lang-typescript`, `sdi-lang-javascript`, `sdi-lang-go`, `sdi-lang-java` crates each with feature gate and `tree-sitter-<lang>` build dep
- Default workspace feature set enables all six languages (matching sdi-py)
- Per-language minimal fixture under `tests/fixtures/simple-<lang>/`
- Multi-language fixture `tests/fixtures/multi-language/` with Python + TypeScript files
- Language detection by extension wired in the file walker

**Files to create or modify:**
- `crates/sdi-lang-{python,typescript,javascript,go,java}/{Cargo.toml,build.rs,src/lib.rs}`
- `crates/sdi-parsing/src/walker.rs` (extension → adapter dispatch table)
- `tests/fixtures/simple-{python,typescript,javascript,go,java}/`
- `tests/fixtures/multi-language/`

**Acceptance criteria:**
- `cargo build --workspace --no-default-features --features lang-python` produces a binary supporting only Python
- `cargo build --workspace` (default features) produces a binary supporting all six
- Each fixture parses to a known `FeatureRecord` count
- Multi-language fixture produces records from both Python and TypeScript files in a single run
- File with extension matching no enabled grammar is skipped with a stderr DEBUG log

**Tests:**
- `tests/full_pipeline.rs` extended: parse each `simple-<lang>/` fixture
- `tests/multi_language.rs`: parse multi-language fixture, assert per-language record counts
- `crates/sdi-parsing/tests/grammar_missing.rs`: build with only `lang-rust`, parse a Python file, assert skip-with-warning behavior

**Watch For:**
- `tree-sitter-typescript` ships two grammars (TSX and TS) — pick one per `.ts` vs `.tsx` extension. Document the choice
- `tree-sitter-go` and `tree-sitter-java` may have outdated crates.io versions; if so, vendor via `[patch.crates-io]` and add a `DRIFT_LOG.md` entry per dependency strategy in DESIGN
- Build times balloon with all six grammars enabled — keep MSRV CI matrix from doubling by caching `~/.cargo` between jobs
- Each adapter's `FeatureRecord` output must be equivalent (under sorted-by-path normalization) to sdi-py's parsing of the same files. **The TS/JS adapter parity is load-bearing** because Milestone 5's `verify-leiden` suite parses bifl-tracker (TypeScript-heavy) through both implementations; an upstream parsing divergence would alias as a Leiden-quality regression. For the other adapters, parity is verified at fixture level only (hand-counted totals in `tests/fixtures/simple-<lang>/`)

**Seeds Forward:**
- The six adapters are the public-facing language support of MVP. Adding a seventh language post-MVP must use this same trait without modification
- The fixture set established here is reused by every subsequent milestone (graph, detection, patterns, snapshot)
- Multi-language fixture is the basis for the verification suite in Milestone 5

---
