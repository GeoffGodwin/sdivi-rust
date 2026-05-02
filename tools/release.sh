#!/usr/bin/env bash
# =============================================================================
# release.sh — sdivi-rust release helper with prepare + post-merge modes
#
# Modes:
#   prepare              Validate version coherence and print a release
#                        checklist. Verifies the workspace version, the
#                        WASM package.json, and the release notes file.
#   post-merge           Create/push tag + create GitHub Release.
#
# Source of truth:
#   Cargo.toml [workspace.package].version (plain MAJOR.MINOR.PATCH)
#
# Adapted from the Tekhton release.sh.
# =============================================================================

set -euo pipefail

MODE="${1:-}"
ARG_VERSION="${2:-}"
COMMIT="${3:-HEAD}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
NOTES_DIR="$REPO_ROOT/tools/release_notes"
CARGO_TOML="$REPO_ROOT/Cargo.toml"
WASM_PKG_JSON="$REPO_ROOT/bindings/sdivi-wasm/package.json"
CHANGELOG="$REPO_ROOT/CHANGELOG.md"

err()  { printf '\033[1;31m[x]\033[0m %s\n' "$*" >&2; exit 1; }
ok()   { printf '\033[1;32m[v]\033[0m %s\n' "$*"; }
info() { printf '\033[1;36m[i]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[!]\033[0m %s\n' "$*"; }

usage() {
    cat <<'EOF'
Usage:
  tools/release.sh prepare [vX.Y.Z] [commit]
  tools/release.sh post-merge [vX.Y.Z] [commit]

Legacy (still supported):
  tools/release.sh vX.Y.Z [commit]        # equivalent to post-merge

Notes:
  - Cargo.toml [workspace.package].version is the source of truth.
  - Optional vX.Y.Z argument must match the workspace version exactly if
    provided.
  - Release notes must exist at tools/release_notes/vX.Y.Z.md.
  - The workspace version must match bindings/sdivi-wasm/package.json.
EOF
    exit 1
}

read_workspace_version() {
    [[ -f "$CARGO_TOML" ]] || err "Cargo.toml missing: $CARGO_TOML"
    local raw
    raw="$(awk '
        /^\[workspace\.package\]/ { in_section = 1; next }
        /^\[/                     { in_section = 0 }
        in_section && /^version[[:space:]]*=/ {
            gsub(/^version[[:space:]]*=[[:space:]]*"/, "")
            gsub(/".*$/, "")
            print
            exit
        }
    ' "$CARGO_TOML")"
    [[ -n "$raw" ]] || err "Could not extract version from $CARGO_TOML"
    [[ "$raw" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || {
        err "[workspace.package].version must be MAJOR.MINOR.PATCH (got: $raw)"
    }
    printf '%s\n' "$raw"
}

read_wasm_pkg_version() {
    [[ -f "$WASM_PKG_JSON" ]] || err "WASM package.json missing: $WASM_PKG_JSON"
    sed -nE 's/^[[:space:]]*"version"[[:space:]]*:[[:space:]]*"([^"]+)".*/\1/p' "$WASM_PKG_JSON" | head -1
}

resolve_version_tag() {
    local from_workspace
    from_workspace="v$(read_workspace_version)"

    if [[ -z "$ARG_VERSION" ]]; then
        printf '%s\n' "$from_workspace"
        return 0
    fi

    [[ "$ARG_VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || {
        err "Version must match vMAJOR.MINOR.PATCH (got: $ARG_VERSION)"
    }

    [[ "$ARG_VERSION" == "$from_workspace" ]] || {
        err "Version mismatch: workspace is $from_workspace but argument is $ARG_VERSION"
    }

    printf '%s\n' "$from_workspace"
}

verify_wasm_version_synced() {
    local workspace_ver wasm_ver
    workspace_ver="$(read_workspace_version)"
    wasm_ver="$(read_wasm_pkg_version)"
    [[ "$workspace_ver" == "$wasm_ver" ]] || {
        err "Version drift: Cargo.toml is $workspace_ver but bindings/sdivi-wasm/package.json is $wasm_ver"
    }
    ok "WASM package.json version matches workspace ($workspace_ver)."
}

verify_changelog_has_entry() {
    local version_tag="$1"
    local plain="${version_tag#v}"
    if grep -qE "^## \[${plain}\]" "$CHANGELOG"; then
        ok "CHANGELOG has entry for [${plain}]."
    else
        warn "CHANGELOG does not have a [${plain}] section. Add one before tagging."
    fi
}

release_notes_path() {
    local version_tag="$1"
    printf '%s/%s.md\n' "$NOTES_DIR" "$version_tag"
}

ensure_commit_exists() {
    local commit_ref="$1"
    if ! git rev-parse --verify "$commit_ref" >/dev/null 2>&1; then
        err "Commit not found: $commit_ref"
    fi
}

ensure_on_main() {
    local current
    current="$(git rev-parse --abbrev-ref HEAD)"
    [[ "$current" == "main" ]] || {
        err "Releases must be cut from main (currently on $current)."
    }

    git fetch origin main >/dev/null 2>&1 || warn "Could not fetch origin/main."
    local local_sha remote_sha
    local_sha="$(git rev-parse main)"
    remote_sha="$(git rev-parse origin/main 2>/dev/null || echo "")"
    if [[ -n "$remote_sha" && "$local_sha" != "$remote_sha" ]]; then
        err "Local main is out of sync with origin/main. Pull first."
    fi
    ok "On main, in sync with origin."
}

run_prepare() {
    local version_tag
    version_tag="$(resolve_version_tag)"

    cd "$REPO_ROOT"

    info "Prepare-release checklist for ${version_tag}:"
    verify_wasm_version_synced
    verify_changelog_has_entry "$version_tag"

    local notes_file
    notes_file="$(release_notes_path "$version_tag")"

    if [[ -f "$notes_file" ]]; then
        ok "Release notes exists: $notes_file"
    else
        warn "Release notes missing: $notes_file"
        printf '     mkdir -p %s\n' "$NOTES_DIR"
        printf '     $EDITOR %s\n' "$notes_file"
    fi

    echo
    info "Next steps:"
    printf '  1. Confirm cargo test --workspace is green.\n'
    printf '  2. Confirm cargo clippy -- -D warnings is clean.\n'
    printf '  3. Confirm CHANGELOG.md [%s] section is final.\n' "${version_tag#v}"
    printf '  4. Merge to main if not already.\n'
    printf '  5. Run: tools/release.sh post-merge %s\n' "$version_tag"
    echo

    [[ -f "$notes_file" ]] || err "Prepare blocked until release notes file exists."

    ok "Prepare step complete."
}

run_post_merge() {
    local version_tag
    version_tag="$(resolve_version_tag)"

    cd "$REPO_ROOT"

    ensure_on_main
    verify_wasm_version_synced
    verify_changelog_has_entry "$version_tag"
    ensure_commit_exists "$COMMIT"

    local commit_sha commit_short commit_subject
    commit_sha="$(git rev-parse "$COMMIT")"
    commit_short="$(git rev-parse --short "$COMMIT")"
    commit_subject="$(git log -1 --format=%s "$COMMIT")"

    if git rev-parse --verify "refs/tags/$version_tag" >/dev/null 2>&1; then
        err "Tag $version_tag already exists locally. Delete it first: git tag -d $version_tag"
    fi
    if git ls-remote --tags origin "$version_tag" 2>/dev/null | grep -q "$version_tag"; then
        err "Tag $version_tag already exists on origin. Aborting to avoid overwriting."
    fi

    local notes_file
    notes_file="$(release_notes_path "$version_tag")"
    [[ -f "$notes_file" ]] || err "Release notes file not found: $notes_file"

    local notes_lines
    notes_lines="$(wc -l < "$notes_file" | tr -d ' ')"
    if [[ "$notes_lines" -lt 5 ]]; then
        warn "Release notes file is only $notes_lines lines. Continue? (Ctrl+C to abort)"
        read -r _
    fi

    info "Release plan:"
    printf '  Version:    %s\n' "$version_tag"
    printf '  Commit:     %s (%s)\n' "$commit_short" "$commit_sha"
    printf '  Subject:    %s\n' "$commit_subject"
    printf '  Notes:      %s (%s lines)\n' "$notes_file" "$notes_lines"
    printf '  Triggers:   .github/workflows/release.yml (manual approval gates for crates.io and npm)\n'
    echo
    read -r -p "Proceed with release? [y/N] " confirm
    [[ "$confirm" =~ ^[Yy]$ ]] || {
        info "Aborted."
        exit 0
    }

    info "Creating annotated tag $version_tag at $commit_short..."
    local tag_message
    tag_message="$(printf 'sdivi-rust %s\n\n' "$version_tag"; cat "$notes_file")"
    git tag -a "$version_tag" "$commit_sha" -m "$tag_message"
    ok "Tag created locally."

    info "Pushing tag to origin..."
    if ! git push origin "$version_tag"; then
        warn "Push failed. The tag exists locally; you can retry with:"
        printf '    git push origin %s\n' "$version_tag"
        exit 1
    fi
    ok "Tag pushed to origin (release.yml workflow will run)."

    if command -v gh >/dev/null 2>&1; then
        info "Creating GitHub Release via gh CLI..."

        local release_title
        release_title="$version_tag — $(head -1 "$notes_file" | sed 's/^#* *//')"

        if gh release create "$version_tag" \
            --title "$release_title" \
            --notes-file "$notes_file"; then
            ok "GitHub Release created. release.yml will attach binaries on completion."
            gh release view "$version_tag" --web 2>/dev/null || true
        else
            warn "gh release create failed. Tag is pushed; create the release manually."
        fi
    else
        warn "gh CLI not found. Tag is pushed but the GitHub Release page must be created manually."
        local repo_url
        repo_url="$(git config --get remote.origin.url | sed -E 's#(git@|https?://)([^:/]+)[:/]([^/]+/[^.]+)(\.git)?#https://\2/\3#')"
        info "Open: ${repo_url}/releases/new?tag=$version_tag"
        info "Paste the body from: $notes_file"
    fi

    echo
    ok "Release $version_tag complete. Watch the release.yml run for the manual approval gates."
}

# Legacy mode: first arg is a version tag
if [[ "$MODE" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    ARG_VERSION="$MODE"
    MODE="post-merge"
fi

case "$MODE" in
    prepare)
        run_prepare
        ;;
    post-merge)
        run_post_merge
        ;;
    *)
        usage
        ;;
esac
