#!/usr/bin/env bash
# publish-or-skip.sh — idempotent wrapper around `cargo publish`.
#
# Used by .github/workflows/release.yml's publish-crates job. The job
# publishes 16 internal crates serially; without this wrapper, a single
# failure (e.g. crates.io's new-crate rate limit, transient 5xx) leaves
# already-published crates in a state where reruns immediately fail at
# the first step ("crate version is already uploaded"), forcing a
# version bump to recover.
#
# This wrapper treats "already uploaded" as success so reruns can resume
# from where the previous run left off. Every other failure mode
# (compile error, auth, true network issue, rate limit) still propagates
# as a non-zero exit and stops the job.
#
# Usage: ./scripts/publish-or-skip.sh <crate-name> [extra cargo publish args]

set -u  # do NOT use `set -e` — we explicitly handle cargo publish's exit code.

if [[ $# -lt 1 ]]; then
    echo "usage: $0 <crate-name> [cargo publish args...]" >&2
    exit 2
fi

crate="$1"
shift

echo "==> cargo publish -p $crate $*"
out=$(cargo publish -p "$crate" "$@" 2>&1)
ec=$?

# Always echo cargo's full output so CI logs are useful.
echo "$out"

if [[ $ec -eq 0 ]]; then
    exit 0
fi

# crates.io's "already uploaded" message is the only failure mode we treat
# as idempotent success. Every other failure (rate limit, auth, network,
# compile error) must still propagate.
if echo "$out" | grep -qE 'crate version .* is already uploaded|already uploaded'; then
    echo "::notice::$crate is already published — skipping (idempotent rerun)"
    exit 0
fi

exit $ec
