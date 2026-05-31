# Reviewer Report — M36.1: `decorators` pattern category (TS/JS)
Review cycle: 1 of 4
Reviewed by: reviewer agent

## Verdict
APPROVED_WITH_NOTES

## Complex Blockers (senior coder)
- None

## Simple Blockers (jr coder)
- None

## Non-Blocking Notes
- `docs/pattern-categories.md` line 185: the dispatch-order summary paragraph reads "P10 (`decorators`) is node-kind-only…" — P10 is already assigned to `collection_pipelines` in the canonical precedence table immediately above (line 180). `decorators` has no slot number in `CALL_DISPATCH`. The sentence should say "The `decorators` category is node-kind-only…" (drop the P10 label).
- `dispatch_disjointness.rs` line 105: the M36.1 corpus entry tests `("@Injectable()", "typescript", "")` as a `call_expression` text. The real inner `call_expression` emitted by tree-sitter inside a decorator node would be `Injectable()` (no `@` prefix) — the `@` belongs to the outer `decorator` node. Both inputs produce `""` so the asserted result is valid, but the comment "A `decorator` hint routed as `call_expression`" is slightly misleading about what text the inner call would carry.
- Double-counting not blocked at the AST-walk level: `collect_hints` in both TS and JS adapters recurses into children of `decorator` nodes, so the inner `call_expression` (e.g. `Injectable()`) is also emitted as a separate hint. For all common decorator callees in NestJS/Angular/TypeORM/class-validator the inner `call_expression` text does not match any current `CALL_DISPATCH` pattern, so no spurious classification occurs in practice. The risk is real for unusual decorator names that happen to match CALL_DISPATCH regexes (e.g. `@fetch()`). Acceptable for v0; the milestone Watch For explicitly flagged this and the practical impact is negligible. Worth a guard comment near the `PATTERN_KINDS` addition in a later cleanup pass.
- Pre-existing: stale assertion message at `crates/sdivi-patterns/src/queries/mod.rs:282` ("logging is catalog-only in v0 for category_for_node_kind") — flagged by M23 and M34 reviewers; not introduced by M36.1. Third milestone carrying this.
- Pre-existing: WASM `package.json` version stranded at 0.2.23 — noted by coder; not introduced by M36.1.

## Coverage Gaps
- No integration test fixture exercises end-to-end parsing of a NestJS-shaped `.ts` file and asserts a non-zero `decorators` instance count. The milestone acceptance criterion "A TS fixture with `@Injectable()`/`@Get()` produces `decorators` instances" is satisfied by unit-level dispatch tests but not by a real tree-sitter parse. A fixture file at `crates/sdivi-lang-typescript/tests/` (or `tests/fixtures/decorator-nestjs/`) running through `collect_hints` and asserting at least one emitted `decorator` hint would close this gap.

## Drift Observations
- `docs/pattern-categories.md:185` — "P10 (`decorators`)" contradicts the canonical precedence table in the same doc (P10 = `collection_pipelines`). Factual error in doc text introduced by M36.1; correct by dropping the slot label.
- `dispatch_disjointness.rs:26` — comment "At M35, P1/P6/P8/P9 are active" not updated to reflect M36.1. Since M36.1 adds no `CALL_DISPATCH` entry the omission is technically accurate, but the milestone marker is now stale.
- `docs/pattern-categories.md` Go corpus — `fmt.Errorf` is classified as `logging` via the `^fmt\.(Print|Println|Printf|Errorf|Fprint|Sprint)` regex. `fmt.Errorf` constructs an error value; it does not emit output. Pre-existing M33 inheritance; the eventual Go error-handling pass will need to revisit this regex entry.
