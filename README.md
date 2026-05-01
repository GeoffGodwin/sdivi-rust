# sdi-rust

**Structural Divergence Indexer** — a measurement instrument that tracks the rate
of structural drift in a codebase over time.

sdi-rust is delivered as a Cargo workspace with a two-layer library shape:

- **`sdi-core`** — pure-compute facade; WASM-compatible, no I/O
- **`sdi-pipeline`** — FS orchestration (parsing, snapshot writes, retention)
- **`sdi-cli`** — thin CLI shell producing the `sdi` binary

## Install

**Cargo (from source)**
```sh
cargo install sdi-cli
sdi --help
```

**Pre-built binaries** (Linux/macOS/Windows — see GitHub Releases)
```sh
# Example: Linux x86_64
curl -Lo sdi https://github.com/geoffgodwin/sdi-rust/releases/latest/download/sdi-linux-x86_64
chmod +x sdi && mv sdi ~/.local/bin/
```

**npm (WASM bundle — ships with 0.1.0)**
```sh
npm install @geoffgodwin/sdi-wasm
```

## Quick Start

```sh
# Initialise the .sdi/ directory in your repo
sdi init

# Capture a snapshot of the current codebase structure
sdi snapshot

# Show the latest snapshot
sdi show

# Run the threshold gate (exits 10 if any threshold is breached)
sdi check
```

## What is SDI?

SDI measures four dimensions of structural health on every merge:

| Dimension | What it tracks |
|---|---|
| Pattern entropy rate | How fast coding patterns are diverging |
| Convention drift rate | How fast style and idiom conventions shift |
| Coupling delta rate | How fast inter-module coupling changes |
| Boundary violation rate | How often code crosses declared module boundaries |

Threshold breaches are observations, not judgements. Teams declare migration
intent via per-category threshold overrides with explicit expiry dates.

## CI Integration

```yaml
- run: sdi check   # exits 0 if healthy, 10 if thresholds exceeded
```

See [`docs/cli-integration.md`](docs/cli-integration.md) for the full GitHub
Actions recipe and exit-code reference.

## Embedding

**Full FS pipeline (Rust)**
```toml
[dependencies]
sdi-pipeline = "0.0.14"
```

**Pure-compute / WASM**
```toml
[dependencies]
sdi-core = "0.0.14"
```

See [`docs/library-embedding.md`](docs/library-embedding.md) for a complete
embedding guide, including the consumer-app pattern where the caller supplies
its own AST extractors.

## Supported Languages

Rust · Python · TypeScript · JavaScript · Go · Java

Languages are detected automatically from file extensions. Additional grammars
are added via feature flags (`lang-rust`, `lang-python`, …).

## Documentation

| Document | What it covers |
|---|---|
| [`docs/cli-integration.md`](docs/cli-integration.md) | CI integration, GHA snippet, exit codes |
| [`docs/library-embedding.md`](docs/library-embedding.md) | Rust and WASM embedding guide |
| [`docs/determinism.md`](docs/determinism.md) | BTreeMap discipline, seed contract, FMA notes |
| [`docs/migrating-from-sdi-py.md`](docs/migrating-from-sdi-py.md) | Migrating from the Python POC |

## Examples

```sh
cargo run --example embed_pipeline    # full FS pipeline
cargo run --example embed_compute     # pure-compute / WASM-path
cargo run --example custom_config     # programmatic Config building
```

## License

Apache 2.0 — see [LICENSE](LICENSE) and [NOTICE](NOTICE).
