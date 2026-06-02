# Drift Log

Records drift between `.tekhton/DESIGN.md` and implementation reality:
vendoring decisions, `[patch.crates-io]` justifications, and design choices
that diverged from the original specification. Entries are added by milestone
authors; entries are never removed (the log is append-only).

---

## M47 — WASM Consumer-Surface Typecheck Guard (2026-06-01)

The M12-era `await init()` docs idiom (designed for `--target web`) survived
the switch to the bundler+nodejs dual build and shipped broken, alongside an
object-literal-where-`Map`-expected example, because no CI step typechecked
the consumer surface against the generated `.d.ts`. The M12 milestone spec
designed for the `--target web` idiom (`import init, { … }; await init()`),
but the implementation sensibly chose `--target bundler` + `--target nodejs`
instead — where the bundler target auto-initialises on import and the nodejs
target loads `.wasm` synchronously at require time, with no init() export in
either case. The docs were not reconciled with this decision; both regressions
(no callable init, object-as-Map) were only caught in a pre-release review.

Closed by:
- A strict `tsc --noEmit` guard (`bindings/sdivi-wasm/tests/typecheck/`) that
  typechecks `examples/binding_node.ts` and `examples/binding_bundler.ts`
  against the freshly built `pkg/*.d.ts` under `--strict
  --noUncheckedIndexedAccess --exactOptionalPropertyTypes`.
- A self-verifying negative fixture (`tests/typecheck/negative.ts`) whose
  `@ts-expect-error` directives lock the exact broken patterns; any contract
  relaxation triggers TS2578 "Unused '@ts-expect-error' directive".
- A forbidden-pattern doc lint (`tests/check_docs.sh`) that greps
  consumer-facing docs and examples for `import init`, `await init(`, and
  `edge_weights: {`, catching prose regressions that `tsc` cannot reach.

Bundler *runtime* e2e remains intentionally deferred to upstream wasm-bindgen
tests + real consumer integration (see M47 Non-Goals and Seeds Forward in the
milestone definition).
