#### Milestone 16: Snapshot at Historical Commit (`--commit REF`)
<!-- milestone-meta
id: "16"
status: "done"
-->


**Scope:** Make `sdi snapshot --commit REF` actually run the pipeline against the tree at REF. Today the flag is plumbed from CLI → `Pipeline::snapshot(commit)` → snapshot metadata as a label, but the parsing stage still reads the working tree, which is misleading. This milestone resolves REF to an immutable SHA, materializes the tree at REF in a tempdir via `git archive | tar -x`, runs the full pipeline against the tempdir, and labels the resulting snapshot with the resolved SHA and the commit's commit-date. M15's change-coupling analyzer is rewired to use `ending_at = REF` so historical snapshots get their proper historical context.

**Why this milestone exists:** CLAUDE.md's "What Not to Build Yet" section currently states `sdi snapshot --commit REF works for individual commits.` That is false today: only the snapshot's `commit` field is populated; the analyzed tree is still the working directory's. Two consequences: (1) historical backfill produces snapshots labeled with one commit but reflecting a different tree, which silently corrupts trend lines; (2) the documented one-shot historical analysis path is unusable, so users have no in-tool way to backfill. With M15 surfacing change-coupling per-snapshot, fixing `--commit REF` is now load-bearing for any consumer that wants accurate temporal data — including Meridian's "show me how this codebase looked at last month's release tag" use case. The change is small, the surface is contained, and shipping v0 with the documented-but-broken flag is an honesty problem more than a scope problem.

**Deliverables:**

- **Ref resolution and tree extraction in `sdi-pipeline`:**
  - `crates/sdi-pipeline/src/commit_extract.rs` (new module):
    - `resolve_ref_to_sha(repo_root: &Path, reference: &str) -> Result<String, CommitExtractError>` — shells out to `git -C <repo_root> rev-parse --verify <reference>`. Returns the full 40-char SHA. Errors: non-zero exit, output not a valid SHA.
    - `commit_date_iso(repo_root: &Path, sha: &str) -> Result<String, CommitExtractError>` — `git -C <repo_root> show -s --format=%cI <sha>`. Returns `YYYY-MM-DDTHH:MM:SS+00:00`-style ISO 8601.
    - `extract_commit_tree(repo_root: &Path, sha: &str) -> Result<TempDir, CommitExtractError>` — pipes `git archive --format=tar <sha>` into `tar -xC <tempdir>`. The pipe is set up via two `std::process::Command` invocations connected through `std::process::Stdio`. The returned `TempDir` cleans up on drop.
    - `CommitExtractError` thiserror enum: `{ RefResolutionFailed { reference, stderr }, CommitNotFound { sha }, ArchiveFailed { stderr }, TarFailed { stderr }, IoError(std::io::Error) }`.
- **Pipeline integration:**
  - `Pipeline::snapshot(repo_root, commit, timestamp)` (existing signature, no change) becomes:
    - When `commit = None`: behavior unchanged. Pipeline runs against `repo_root`. Snapshot's `commit` field is `None`.
    - When `commit = Some(reference)`:
      1. `sha = resolve_ref_to_sha(repo_root, reference)?`
      2. `commit_date = commit_date_iso(repo_root, &sha)?`
      3. `tempdir = extract_commit_tree(repo_root, &sha)?`
      4. The provided `timestamp` argument is **overridden** by `commit_date` for snapshot file naming and `Snapshot.timestamp` (so trend ordering tracks chronology, not wall-clock-of-CLI-invocation).
      5. The five-stage pipeline runs against `tempdir.path()` (parsing, graph, detection, patterns, snapshot assembly).
      6. M15's change-coupling collection runs against the **original `repo_root`**, not the tempdir (the tempdir has no `.git/`). `collect_cochange_events` is called with `ending_at = Some(&sha)` so the analyzed window is the `history_depth` commits ending at REF, not at HEAD.
      7. The Leiden warm-start cache continues to read/write `<repo_root>/.sdi/cache/partition.json` (not the tempdir). Cache reuse across `--commit` invocations is a feature, not a bug.
      8. Snapshot file is written to `<repo_root>/.sdi/snapshots/` (not the tempdir's). Atomic write + retention enforcement unchanged.
      9. `Snapshot.commit = Some(sha)` (the resolved SHA, not the user-supplied ref name).
      10. `tempdir` drops at end of scope, removing the materialized tree.
  - `Pipeline::snapshot_with_mode` follows the same logic; `WriteMode::EphemeralForCheck` from M09 still applies (no snapshot persisted, no cache write).
- **Documentation:**
  - `docs/cli-integration.md` — new section "Analyzing a historical commit" describing `sdi snapshot --commit REF`, what the flag does, what gets persisted (the snapshot is a real persisted snapshot named after the commit-date timestamp), and the interaction with change-coupling history.
  - `docs/library-embedding.md` — note that the pure-compute path (`sdi-core`) does not need any of this — embedders that already have their own tree extraction (the consumer app, Meridian) call `compute_*` directly with whatever tree they have in hand. `--commit REF` is an `sdi-pipeline` / CLI convenience.
  - `CLAUDE.md` — the line `Historical backfill UX — sdi snapshot --commit REF works for individual commits. Batch backfill is unsupported; users script it.` in "What Not to Build Yet" remains accurate after this milestone (now genuinely so). The user is responsible for confirming the line still matches reality post-merge; no milestone-time edit.
- **CHANGELOG.md** entry: "`sdi snapshot --commit REF` now analyzes the actual tree at REF. The snapshot is labeled with the resolved SHA and the commit's commit-date (not wall-clock time). Change-coupling history is computed ending at REF. Pre-v0 callers relying on the prior label-only behavior must adjust."
- **Error UX:**
  - `--commit nonexistent` → `CommitExtractError::RefResolutionFailed`, CLI exit 1 with a stderr message naming the unresolvable reference.
  - `--commit` against a non-git directory → `RefResolutionFailed` (git itself errors). Same exit 1 path.
  - `--commit` against a shallow clone where REF is below the shallow boundary → `git rev-parse` succeeds but `git archive` may fail; surface as `ArchiveFailed`.
  - All error paths include the captured `stderr` from the failing `git` invocation so users see git's actual diagnostic.

**Files to create or modify:**

- **New:** `crates/sdi-pipeline/src/commit_extract.rs` — `resolve_ref_to_sha`, `commit_date_iso`, `extract_commit_tree`, `CommitExtractError`.
- **Modify:** `crates/sdi-pipeline/src/error.rs` (or wherever `PipelineError` is defined) — add `CommitExtract(CommitExtractError)` variant with `#[from]`.
- **Modify:** `crates/sdi-pipeline/src/pipeline.rs` — branch `Pipeline::snapshot_with_mode` on `commit.is_some()`; call the new extraction helpers; pass tempdir as the parsing root; pass `ending_at = Some(&sha)` to `collect_cochange_events`; override `timestamp` with `commit_date_iso` output.
- **Modify:** `crates/sdi-pipeline/src/lib.rs` — re-export `commit_extract` module (probably `pub(crate)` unless an embedder needs the helpers directly; expose at minimum `CommitExtractError` for downstream error matching).
- **Modify:** `crates/sdi-cli/src/commands/snapshot.rs` — error formatting for the new `PipelineError::CommitExtract` variant; CLI-level integration test fixture.
- **New:** `crates/sdi-pipeline/tests/commit_snapshot.rs` — fixture git repo with three commits; snapshot at HEAD, HEAD~1, HEAD~2; assert each is distinct, assert SHA labeling, assert commit-date timestamping, assert tempdir cleanup.
- **New:** `tests/historical_commit_lifecycle.rs` (workspace-level) — full CLI invocation via `assert_cmd`: `sdi snapshot --commit HEAD~1` against a built fixture, parse snapshot JSON, verify expected fields, verify file naming under `.sdi/snapshots/`.
- **Modify:** `crates/sdi-cli/tests/exit_codes.rs` — add cases for `--commit nonexistent` (exit 1) and `--commit` in a non-git directory (exit 1).
- **Modify:** `tools/build-test-fixtures.sh` — extend to build a `historical-commits/` fixture: a tempdir git repo with three known commits each touching a known file set. Built into `target/test-fixtures/historical-commits/` like the existing `evolving/` fixture.
- **Modify:** `crates/sdi-pipeline/Cargo.toml` — `tempfile` is already a dev-dep; promote to a runtime dep (it's now used in `extract_commit_tree`'s production path). No new external deps.
- **Modify:** `docs/cli-integration.md`, `CHANGELOG.md`.

**Acceptance criteria:**

- `sdi snapshot --commit HEAD~1` against the fixture (a 3-commit repo where each commit adds a distinct file) produces a snapshot whose graph node count matches the file count *as of HEAD~1*, not HEAD.
- The snapshot's `commit` field is the full 40-char SHA, not the ref name `HEAD~1`.
- The snapshot's `timestamp` field is the commit's commit-date in ISO 8601 form, not the wall-clock time of the CLI invocation.
- The snapshot file's name (under `.sdi/snapshots/`) reflects the commit-date timestamp, so lexicographic sort matches chronological sort across mixed `--commit` and HEAD invocations.
- The Leiden warm-start cache at `<repo_root>/.sdi/cache/partition.json` is read and written normally (not isolated to the tempdir).
- The change-coupling section (M15) of the snapshot reflects the `history_depth` commits ending at the resolved SHA, not ending at HEAD.
- `tempdir` is dropped before `Pipeline::snapshot` returns; a test that captures the tempdir path during execution and re-checks after asserts the path no longer exists.
- `sdi snapshot --commit nonexistent-ref` exits with code 1 and a stderr message including git's actual `rev-parse` error.
- `sdi snapshot --commit HEAD` (with no other changes) produces a snapshot byte-identical to `sdi snapshot` with no `--commit` flag, *except* that the `commit` field carries the resolved SHA where the no-flag path leaves it `None`, and the `timestamp` is the commit-date instead of wall-clock. (This asymmetry is the intended UX — users who want labeling can pass `--commit HEAD`.)
- A second invocation of `sdi snapshot --commit <sha>` with the same SHA produces a byte-identical snapshot file (modulo the file's mtime), verifying determinism across re-runs.
- The M11 bifl-tracker harness still passes within tolerances against HEAD-based snapshots; a new sub-step picks one historical commit from the bifl-tracker baseline set, runs `sdi snapshot --commit <historical-sha>`, and verifies the per-dimension metrics match the sdi-py-era snapshot for that same commit (within the same KDD-2 tolerances). This is the v0 acceptance gate for "historical backfill is real."
- `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` pass.
- `cargo test --workspace` passes including doctests.
- `snapshot_version` remains `"1.0"`.

**Tests:**

- Integration: `crates/sdi-pipeline/tests/commit_snapshot.rs` covers the three-commit fixture across HEAD/HEAD~1/HEAD~2.
- Integration: `tests/historical_commit_lifecycle.rs` exercises the full CLI invocation.
- Negative: `--commit nonexistent` → exit 1.
- Negative: `--commit` in a dir with no `.git/` → exit 1 (git emits a `not a git repository` error; surface it through `RefResolutionFailed`).
- Determinism: two back-to-back invocations of `--commit <sha>` produce byte-identical snapshot JSON.
- Cleanup: tempdir path captured during run is gone after `Pipeline::snapshot` returns. (Test technique: `TempDir::into_path` is *not* called; the test retrieves the path before drop and asserts post-drop non-existence — but since `tempdir` is owned inside the pipeline, the test instead replaces extraction with a hookable variant or asserts via `/proc/self/fd` / open-handle inspection. Pragmatic alternative: count `target/sdi-extract-*` directories before and after; assert delta = 0.)
- Change-coupling-at-REF: fixture with a known co-change history; snapshot at HEAD~1 produces a `change_coupling` section reflecting commits up to HEAD~1, *not* including the most recent commit.
- Cross-platform: the integration test runs on Linux, macOS, and Windows in CI. Windows requires `git` and `tar` (or the `git archive --format=zip` fallback — see Watch For).

**Watch For:**

- **Windows `tar` availability.** Default Windows 10/11 ships `tar.exe` in `System32` since 2017, but minimal CI runner images may not have it on `PATH`. Mitigation order: (1) check for `tar` via `Command::new("tar").arg("--version")` at the top of `extract_commit_tree`; (2) if absent, fall back to `git archive --format=zip` piped to a Rust-side zip extractor. For v0, prefer mitigation (1) only; if `tar` is missing, return a structured error advising the user. Document the requirement in `docs/cli-integration.md`. The `windows-latest` GitHub runner has `tar` available — verify in CI.
- **Pipe wiring.** `git archive --format=tar <sha> | tar -xC <tempdir>` is two processes. In Rust: spawn `git archive` with `Stdio::piped()` for stdout, spawn `tar` with `Stdio::piped()` for stdin, copy bytes via `std::io::copy` (or pass the file handle directly via `Stdio::from(child.stdout)`). Test: archive a commit with 1MB+ of files, ensure the pipe doesn't deadlock on a small kernel buffer.
- **`.gitattributes export-ignore`.** `git archive` honors `.gitattributes export-ignore` directives. Files marked `export-ignore` are excluded from the archive. This may surprise users who expect "everything tracked at this commit" — document. Acceptance test: a fixture with an `export-ignore`'d file and a non-ignored file; the snapshot's graph excludes the ignored file. (This is *correct* behavior — but it must be documented because it diverges from `git checkout`'s semantics.)
- **Shallow clones.** If `repo_root` is a shallow clone and REF is below the shallow boundary, `git rev-parse` succeeds but `git archive` errors. The error is surfaced cleanly via `ArchiveFailed`. CI runners often produce shallow clones by default; the integration tests should use `actions/checkout@v4` with `fetch-depth: 0` for milestones touching git history.
- **Submodules.** `git archive` does not recurse into submodules. A snapshot at REF with submodule references will not include submodule contents. Document. (Same caveat as M15.)
- **Symlinks.** `git archive | tar -x` preserves symlinks. The parsing stage's `walkdir + ignore` already handles symlinks correctly. No special handling needed.
- **Path canonicalization.** When the tempdir replaces `repo_root`, `walkdir` yields paths under the tempdir. The `FeatureRecord` machinery already produces NodeIds via the existing canonicalizer, which strips the prefix and emits repo-relative paths. Verify: a snapshot at HEAD~1 has the same `NodeInput.id` strings as a snapshot at HEAD for files that exist in both — this is what makes the partition-cache warm-start meaningful across `--commit` invocations.
- **Cache poisoning across `--commit` invocations.** The Leiden warm-start cache is keyed by the partition's structure, not by file content. If commit A and commit B have very different file sets, warming from A's partition into B's run still works (Leiden converges from any starting partition; warm-start is an optimization, not a correctness primitive). Document.
- **Concurrency.** Two simultaneous `sdi snapshot --commit ...` invocations against the same `repo_root` would race on `.sdi/snapshots/` and `.sdi/cache/partition.json`. The atomic write rule (Rule 9) protects the snapshot files; the cache write is non-atomic and the last writer wins. This is acceptable for v0 (the snapshot is the durable artifact; the cache is regenerable). Document.
- **`commit_date_iso` timezone.** `git show -s --format=%cI` emits committer-date in the committer's timezone (e.g., `2026-04-30T14:23:01-07:00`). For deterministic snapshot file naming and trend ordering, normalize to UTC at this layer (call `.with_timezone(&Utc)` after parsing, then re-emit). Cross-platform tests must lock this down.
- **`Pipeline::snapshot`'s `timestamp` parameter is now usually overridden.** Existing callers passing a wall-clock timestamp expect it to land in the snapshot. With M16, when `commit` is `Some`, the supplied `timestamp` is silently replaced by `commit_date_iso`. This is intentional — historical snapshots labeled with wall-clock time poison trend ordering — but it's a behavior change. The `Pipeline::snapshot` rustdoc must document this, and the CHANGELOG entry must call it out.
- **CLAUDE.md.** No edit required from this milestone — the line `sdi snapshot --commit REF works for individual commits` is now true. The user should confirm post-merge that the broader Rule-set and Critical-System-Rules sections don't need an additional line about commit-date timestamp normalization.
- **Doc-comment placement.** Standard caveat from M14/M15.

**Seeds Forward:**

- `--commit REF` is the foundation for any future batch-backfill UX (`sdi snapshot --commit-range A..B` or `sdi backfill`). Such a UX iterates `git rev-list A..B` and calls `Pipeline::snapshot` for each — the per-commit cost is dominated by parsing, which is cached implicitly via the partition warm-start across runs. The "What Not to Build Yet" line on batch backfill stays in force for v0; M16 just makes the per-commit primitive solid.
- The `commit_extract` module is the natural home for any future git-touching helpers (e.g., `current_branch`, `is_dirty_working_tree`). Keep it small in v0.
- A v0.x can introduce a `--ephemeral-commit REF` mode (snapshot computed but not persisted) that pairs with M09's `WriteMode::EphemeralForCheck`. Out of scope for M16; the primitive is already there.
- Bare-repository support: with M16's tempdir extraction, `sdi snapshot --commit REF` could in principle run against a bare repo (no working tree). Not a v0 promise; document as "may work, not tested."

---
