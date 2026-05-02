# Migration notes

Breaking-change migration guidance for sdivi-rust adopters. Each `0.x → 0.(x+1)`
bump that touches stable surface gets an entry. Post-1.0, the same applies to
major-version bumps.

For the broader migration story from the Python POC (`sdi-py`), see
[`docs/migrating-from-sdi-py.md`](docs/migrating-from-sdi-py.md).

## 0.1.x

No breaking changes between 0.1.0 and 0.1.8. Every release in the 0.1 line is
backwards-compatible at the public-API and snapshot-schema level. New `Input`
fields are added with `#[serde(default)]` and new snapshot fields are
additive.

The 0.1.7 algorithm correction in the Leiden refinement phase is not a public
API break. It does invalidate trend continuity across the 0.1.6 / 0.1.7
boundary because pre-0.1.7 snapshots have a `modularity` value derived from
the broken refinement. See `CHANGELOG.md` 0.1.7 entry.

## Future entries

When a breaking change lands, document:

- **What changed.** A precise description of the renamed, removed, or
  resemanticised item.
- **Why.** The motivation. Often a correctness fix or a SemVer-mandated
  cleanup.
- **What to do.** A concrete migration recipe. A diff or `sed` snippet
  beats prose.
- **Trend continuity.** Whether snapshots from prior versions are still
  comparable.
