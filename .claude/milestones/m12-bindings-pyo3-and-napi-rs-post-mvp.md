#### Milestone 12: Bindings — PyO3 and napi-rs (Post-MVP)

**Scope:** Re-expose `sdi-core` through PyO3 (`bindings/sdi-py`) and napi-rs (`bindings/sdi-node`). Both ship in this workspace per KDD-11. Python wheel to PyPI; Node prebuilt to npm. Bindings mirror the Rust surface idiomatically: `sdi.Pipeline(cfg).snapshot(path)` in both languages.

**Deliverables:**
- `bindings/sdi-py/` PyO3 crate exposing `Pipeline`, `Config`, `Snapshot`, `DivergenceSummary`
- `bindings/sdi-node/` napi-rs crate exposing the same API
- Python wheel build matrix (Linux/macOS/Windows × Python 3.9–3.13); PyPI publish in release workflow
- Node prebuilt matrix; npm publish
- `examples/binding_python.py` runnable
- `docs/library-embedding.md` extended with binding usage sections

**Files to create or modify:**
- `bindings/sdi-py/{Cargo.toml,pyproject.toml,src/lib.rs}`
- `bindings/sdi-node/{Cargo.toml,package.json,src/lib.rs}`
- `examples/binding_python.py`
- `.github/workflows/release.yml` extended with PyPI and npm publish jobs
- `docs/library-embedding.md` (extend)

**Acceptance criteria:**
- `pip install sdi-py` + `python -c "import sdi; print(sdi.Pipeline(sdi.Config()).snapshot('.'))"` works
- `npm install sdi-node` + an equivalent Node script works
- Bindings produce snapshot JSON identical to the Rust API output for the same input
- Wheels and prebuilts ship for all supported platforms

**Tests:**
- `bindings/sdi-py/tests/test_basic.py`: build wheel locally, run pytest
- `bindings/sdi-node/tests/basic.test.js`: jest or node:test
- A cross-binding test: same fixture → identical JSON via Rust, Python, Node

**Watch For:**
- PyO3 GIL: keep the snapshot computation `Send` and release the GIL during long-running ops via `py.allow_threads`
- napi-rs: avoid blocking the Node event loop on long snapshots; use `napi::Task` or `tokio_threadpool`
- Path conversions: Python's `Path` and Node's `string` paths convert to `&Path` — use `AsRef<Path>` bounds and document Windows path edge cases
- The bindings crate's `unsafe` is allowed only as required by binding macros — Rule 1 stays in force

**Seeds Forward:**
- Bindings are now part of the release matrix. Future MVP+1 features must consider all three surfaces (Rust, Python, Node)
- Binding API stability: bindings inherit `sdi-core`'s SemVer; a breaking change in `sdi-core` propagates to PyPI/npm major bumps
- WASM (KD14) remains deferred until a real consumer surfaces
