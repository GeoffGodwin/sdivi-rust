# SDI Configuration

SDI is configured via `.sdi/config.toml` (TOML), with `.sdi/boundaries.yaml`
holding the optional boundary spec. Both files are **read-compatible with
sdi-py** — drop-in migration of user-edited config is supported.

## Precedence (highest to lowest)

1. Function arguments (library) / CLI flags (binary)
2. Env vars: `SDI_LOG_LEVEL`, `SDI_WORKERS`, `SDI_CONFIG_PATH`, `SDI_SNAPSHOT_DIR`, `NO_COLOR`
3. Project-local `.sdi/config.toml`
4. Global `$XDG_CONFIG_HOME/sdi/config.toml` (fallback `~/.config/sdi/config.toml`)
5. Built-in defaults

`Config::load_or_default(repo_root)` resolves the chain. **All keys are
optional** — missing keys fall through to defaults.

## Merge strategy

- Section-by-section override; later wins.
- Within a section, key-by-key override.
- Lists in `core.exclude` and `patterns.scope_exclude` are **replaced**, not
  merged.
- Each `[thresholds.overrides.<cat>]` block replaces the prior one wholesale
  for that category.

## Complete defaults

```toml
[core]
languages = "auto"
exclude = [
  "**/vendor/**", "**/node_modules/**", "**/__pycache__/**",
  "**/dist/**", "**/build/**", "**/target/**", "**/.git/**",
]
random_seed = 42

[snapshots]
dir = ".sdi/snapshots"
retention = 100        # 0 = unlimited

[boundaries]
spec_file = ".sdi/boundaries.yaml"
leiden_gamma = 1.0     # manual override only — no auto-tuning
stability_threshold = 3
weighted_edges = false

[patterns]
categories = "auto"
min_pattern_nodes = 5
scope_exclude = []     # excludes from catalog only — files remain in graph

[thresholds]
pattern_entropy_rate = 2.0
convention_drift_rate = 3.0
coupling_delta_rate = 0.15
boundary_violation_rate = 2.0

[change_coupling]
min_frequency = 0.6
history_depth = 500

[output]
format = "text"        # "text" | "json"
color = "auto"         # "auto" | "always" | "never"

[determinism]
enforce_btree_order = true   # sdi-rust-only

[bindings]
# reserved for binding-specific knobs (post-MVP)
```

## Per-category threshold overrides

Used when a team is intentionally migrating a pattern category and wants the
gate to tolerate elevated entropy until the migration completes.

```toml
[thresholds.overrides.error_handling]
pattern_entropy_rate = 5.0
expires = "2026-09-30"           # MANDATORY — ISO 8601 date
reason = "Migrating to ? operator from `match Err(_)` chains"
```

### Rules that bite

- **`expires` is mandatory.** Missing it returns
  `ConfigError::MissingExpiresOnOverride { category }`, exit code `2`.
- **After expiry the override is silently ignored** — defaults resume. No manual
  reset, no retention. Run the gate against current expectations again.
- **Unknown keys** elsewhere in config produce a stderr deprecation warning but
  never error. Once a key is introduced, it's reserved forever.
- **Invalid values** (out-of-range numbers, malformed dates) return
  `ConfigError::InvalidValue { key, message }`, exit code `2`.

## Boundary spec — `.sdi/boundaries.yaml`

Read with `serde_yaml`. Schema is identical to sdi-py's. **Missing file is
normal operation** — no warning is emitted; intent divergence is simply absent.

Programmatic writes via `sdi boundaries ratify` may regress comment
preservation. This is an accepted MVP limitation (KDD-6); user-edited comments
in `boundaries.yaml` may not survive a ratify cycle. Document hand-maintained
sections accordingly.

Subcommands:

- `sdi boundaries infer` — propose a boundary spec from current Leiden partition
- `sdi boundaries ratify` — write the proposed spec to `.sdi/boundaries.yaml`
- `sdi boundaries show` — print the current spec

## Runtime mutability

`Config` is consumed at `Pipeline::new`. **The pipeline does not mutate config
during a snapshot run.** Per-call overrides build a new `Config`. There is no
global mutable config in `sdi-core`.
