# Releasing sdivi-rust

The release pipeline is tag-driven. Pushing a `v<MAJOR>.<MINOR>.<PATCH>` tag
to `main` triggers `.github/workflows/release.yml`, which:

1. Verifies the tag matches `Cargo.toml`'s workspace version.
2. Runs `cargo audit` against the lockfile.
3. Builds release binaries for Linux x86_64 + aarch64, macOS x86_64 +
   aarch64, and Windows x86_64.
4. Creates a GitHub Release and attaches all binaries. Auto-generated
   release notes pull from merged PRs since the previous tag.
5. Waits for manual approval at the `crates-io-publish` environment, then
   publishes the 11 workspace crates to crates.io in dependency order.
6. Waits for manual approval at the `npm-publish` environment, then
   publishes `@geoffgodwin/sdivi-wasm` to npm.

## Pre-release checklist

Before tagging, confirm:

- [ ] All open milestones for the release are merged into `main`.
- [ ] `cargo test --workspace` is green on `main`.
- [ ] `cargo clippy -- -D warnings` and `cargo fmt --check` are green.
- [ ] `cargo doc --workspace --no-deps` produces zero warnings.
- [ ] `CHANGELOG.md` has an entry for the new version with a release date.
   Move any items from `[Unreleased]` into the new version's section.
- [ ] `Cargo.toml` workspace version is bumped.
- [ ] `bindings/sdivi-wasm/package.json` version matches the workspace
   version. CI verifies this.
- [ ] `Cargo.lock` is up to date (`cargo update -w` if needed).
- [ ] If a public API was renamed or removed, `MIGRATION_NOTES.md` has an
   entry.
- [ ] If a `pub` item was added, it has a doc comment and an `# Examples`
   block where meaningful (`#![deny(missing_docs)]` will catch this on
   `sdivi-core`).

## Cut a release

```sh
# 1. Confirm CI is green on the tip of main.
gh run list --branch main --limit 5

# 2. Bump the workspace version (single source of truth).
$EDITOR Cargo.toml             # update [workspace.package].version
$EDITOR bindings/sdivi-wasm/package.json   # match the same version
cargo update -w                # refreshes Cargo.lock

# 3. Move CHANGELOG entries from [Unreleased] into [<NEW_VERSION>].
$EDITOR CHANGELOG.md

# 4. Commit and push.
git commit -am "chore(release): prepare v<NEW_VERSION>"
git push

# 5. Tag and push the tag.
git tag -a v<NEW_VERSION> -m "v<NEW_VERSION>"
git push origin v<NEW_VERSION>
```

## Approve the publishes

The release workflow stops twice for manual approval:

1. **`crates-io-publish` environment.** Approve in the GitHub Actions UI to
   publish the 11 crates to crates.io. Order is enforced; do not rerun
   individual jobs.
2. **`npm-publish` environment.** Approve in the GitHub Actions UI to
   publish `@geoffgodwin/sdivi-wasm` to npm.

If either approval is rejected, fix the issue locally, push a new commit,
and tag a new patch release. Do not delete and re-push tags.

## Pre-release tags

Pre-release tags follow `v<MAJOR>.<MINOR>.<PATCH>-rc.<N>` (e.g.
`v0.2.0-rc.1`). Pre-releases:

- Skip the npm publish (`dry-run` only).
- Mark the GitHub Release as a pre-release.
- Still publish to crates.io (cargo accepts pre-release versions natively).

## Yanking a release

If a release is broken:

1. Yank the affected crates: `cargo yank --version <X.Y.Z> -p <crate>`.
2. Mark the GitHub Release as a draft (or delete it if you have not yet
   announced).
3. Cut a new patch release with the fix.

`cargo yank` does not unpublish. Crates remain downloadable for users with
the version pinned, which is the intended behaviour. Yanking blocks new
dependents from picking the version.
