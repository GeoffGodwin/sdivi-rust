# Migrating from sdi-py to sdi-rust

> **Status:** Stub — full migration guide completed in Milestone 11.
> This file covers the YAML comment-loss behaviour (M10 deliverable).
> Additional sections (snapshot schema, config keys, exit codes) will be
> added when M11 lands.

## YAML Comment Loss on `sdi boundaries ratify`

### What changed

`sdi boundaries ratify` writes `.sdi/boundaries.yaml` programmatically using
`serde_yml`. This means **all YAML comments are lost** whenever ratify
overwrites the file.

### What you will see

The first time `sdi boundaries ratify` overwrites a file that contains YAML
comments, it prints a warning to stderr:

```
sdi: warning: '.sdi/boundaries.yaml' contains YAML comments — comments will be
lost after ratify (see docs/migrating-from-sdi-py.md)
```

The command still succeeds (exit 0). The comment-stripped version is written
atomically.

### Why this happens

Comment-preserving YAML round-trips require a hand-written emitter or an
immature crate; neither is acceptable for the MVP quality bar (KDD-6). The
decision can be revisited after v1.0 based on user feedback.

### Workarounds

- **Keep comments in a separate doc.** Move long explanations to a
  `docs/boundaries-rationale.md` and link them from `allow_imports_from`
  entries or boundary names.
- **Do not run `ratify` on a hand-edited file.** Use `ratify` only on fresh
  inference output; edit the resulting file by hand without running ratify
  again.
- **Version-control the file.** With git history, deleted comments can always
  be recovered.

### What is NOT affected

- Manual edits to `.sdi/boundaries.yaml` are preserved between snapshots
  (ratify is not run automatically).
- `.sdi/config.toml` is never written programmatically by sdi-rust.
