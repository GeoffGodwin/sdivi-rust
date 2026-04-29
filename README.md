# sdi-rust

**Structural Divergence Indexer** — a measurement instrument that tracks the rate
of structural drift in a codebase over time. Delivered as a Cargo workspace with
a `sdi-core` library crate and a `sdi-cli` binary crate producing the `sdi` command.

## Install

```sh
cargo install sdi-cli
sdi --help
```

## Quick Start

```sh
# Initialise the .sdi/ directory in your repo
sdi init

# Capture a snapshot of the current codebase structure
sdi snapshot

# Compare against the previous snapshot
sdi diff

# Run the threshold gate (exits 10 if any threshold is breached)
sdi check
```

## What is SDI?

SDI measures five dimensions of structural health on every merge:

| Dimension | What it tracks |
|---|---|
| Pattern entropy rate | How fast coding patterns are diverging |
| Convention drift rate | How fast style and idiom conventions shift |
| Coupling delta rate | How fast inter-module coupling changes |
| Boundary violation rate | How often code crosses declared module boundaries |

Threshold breaches are reported as observations, not judgements. Teams declare
migration intent via per-category threshold overrides with explicit expiry dates.

## Embedding

Add `sdi-core` to your `Cargo.toml` to run the analysis pipeline programmatically:

```toml
[dependencies]
sdi-core = "0.0.0"
```

See [`examples/embed_pipeline.rs`](examples/embed_pipeline.rs) for a minimal
embedder example.

## License

Apache 2.0 — see [LICENSE](LICENSE) and [NOTICE](NOTICE).
