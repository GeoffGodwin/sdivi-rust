#### Milestone 11: Release Pipeline and Distribution

**Scope:** Ship the release. Tag-driven workflow publishes affected crates to crates.io behind a manual approval gate, attaches matrix-built binaries to GitHub Releases. `cargo audit` weekly. Cut `0.1.0`.

**Deliverables:**
- `.github/workflows/release.yml` with manual approval gate before crates.io push
- `cargo dist` or hand-rolled matrix build (Linux x86_64+aarch64, macOS x86_64+aarch64, Windows x86_64) attaching stripped binaries with LTO to GitHub Release
- `CHANGELOG.md` with `0.1.0` entry covering all milestones
- `cargo audit` weekly cron in `.github/workflows/audit.yml`
- Binary size tracked in `CHANGELOG.md` per release
- crates.io publish (in dependency order): `sdi-config`, six `sdi-lang-*`, `sdi-parsing`, `sdi-graph`, `sdi-detection`, `sdi-patterns`, `sdi-snapshot`, `sdi-core`, `sdi-cli`, `sdi-rust` (the install-discovery meta-crate). Each with version `0.1.0`. Bindings (`sdi-py`, `sdi-node`) are not published in this milestone — they ship in Milestone 12 (post-MVP)

**Files to create or modify:**
- `.github/workflows/release.yml` (full)
- `CHANGELOG.md` (0.1.0 entry)
- `Cargo.toml` (workspace version bump to `0.1.0`)
- Each crate's `Cargo.toml` populated with `description`, `repository`, `license = "Apache-2.0"`, `readme`, `keywords`, `categories`

**Acceptance criteria:**
- Tagging `v0.1.0` triggers the release workflow; crates.io push waits on manual approval
- After approval, `cargo install sdi-rust` from crates.io succeeds and produces a working `sdi` binary on PATH (binary name comes from `[[bin]] name = "sdi"` in `sdi-cli`)
- GitHub Release page has binaries for all five Tier-1 + Tier-2 platforms
- Binary size noted in CHANGELOG
- `cargo audit` cron green

**Tests:**
- Dry-run the release workflow on a `vX.Y.Z-rc.N` pre-tag
- Smoke test: `cargo install --version 0.1.0 sdi-rust` on each platform; `sdi --version` reports `0.1.0`
- `cargo audit` clean

**Watch For:**
- crates.io is append-only — no yanking-as-rollback; once `0.1.0` is published it stays. Validate carefully via the dry run
- Manual approval gate must be enforced — auto-publish on tag is explicitly rejected by DESIGN
- LICENSE in each crate's metadata must say `Apache-2.0`, matching the repo LICENSE
- Strip + LTO bloat fix: `[profile.release] lto = "thin"`, `strip = true`, `panic = "abort"` (the last only if no test code unwinds)

**Seeds Forward:**
- `0.1.0` is the SemVer commitment baseline. Adding `pub` items is now deliberate; removing them requires a major bump to `1.0.0`
- The release workflow is reused for every subsequent tag
- Binary distribution channels (crates.io + GitHub Releases) are the public commitments. Adding PyPI/npm in Milestone 12 must not regress these

---
