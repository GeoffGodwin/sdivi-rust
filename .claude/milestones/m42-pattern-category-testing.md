
#### Milestone 42: Pattern Category — `testing`

<!-- milestone-meta
id: "42"
status: "planned"
-->

**Scope:** Add the `testing` category for test-suite structure and assertions.
TS/JS: Jest/Vitest/Mocha (`describe`, `it`, `test`, `expect`, `beforeEach`,
`afterEach`, `jest.mock`, `vi.fn`, `vi.mock`). Python: pytest/unittest
(`assert*`-style via `self.assert...`, plus pytest is largely decorator/bare-assert
based — partial coverage). Go: `t.Run`, `t.Error`, `t.Fatal`. Callee-text on
`call_expression`; depends on M34. No parsing-layer change.

**Why this milestone exists:** Test-structure conventions (BDD `describe/it` vs flat
`test`, mocking style, assertion style) drift notably and are worth measuring
separately. High prevalence in any maintained codebase.

**Deliverables:**
- Create `crates/sdivi-patterns/src/queries/testing.rs` with per-language regex.
- Register in `ALL_CATEGORIES`, M34 `CALL_DISPATCH` at **slot P2 (just below
  `async_patterns`)** — test globals are specific and should resolve before any
  broader category. Add `CATALOG_ENTRIES`.
- Update `docs/pattern-categories.md`.
- **Decide and document the `scope_exclude` interaction** (see Watch For) — this is
  the load-bearing design choice of the milestone.

**Detection (finalized — `^`-anchored test globals + framework-namespaced helpers):**
| Language | Pattern | Examples matched |
|---|---|---|
| TS / JS | `^(describe\|it\|test\|xit\|xdescribe\|fdescribe\|fit\|context\|beforeEach\|afterEach\|beforeAll\|afterAll\|expect)\(` | `describe('x', ...)`, `it('does', ...)`, `expect(y).toBe(z)` |
| TS / JS | `^(jest\|vi)\.(fn\|mock\|spyOn\|clearAllMocks\|resetAllMocks\|useFakeTimers)\(` | `vi.fn()`, `jest.mock('./m')` |
| Go | `\bt\.(Run\|Error\|Errorf\|Fatal\|Fatalf\|Helper\|Skip\|Skipf\|Log\|Logf\|Cleanup\|Parallel)\(` | `t.Run("c", ...)`, `t.Fatal(err)` |
| Python | `\bself\.assert[A-Z]\w*\(` | `self.assertEqual(a, b)`, `self.assertTrue(x)` |

**Migration Impact:** Additive; `list_categories()` +1. New bucket — but **only if
test files are in scope.** Many repos exclude tests via `patterns.scope_exclude`,
in which case the bucket is empty and the migration is a no-op. Document the
dependency on config. `snapshot_version` stays `"1.0"`. `MIGRATION_NOTES.md` entry.

**Files to create or modify:**
- **Create:** `crates/sdivi-patterns/src/queries/testing.rs`.
- **Modify:** `crates/sdivi-patterns/src/queries/mod.rs`,
  `crates/sdivi-core/src/categories.rs`.
- **Modify:** `docs/pattern-categories.md`, `MIGRATION_NOTES.md`, `CHANGELOG.md`.

**Acceptance criteria:**
- `expect(x).toBe(1)` → `["testing"]`; `describe('s', fn)` → `["testing"]`.
- A non-test `test(args)` business call is an accepted false-positive (documented).
- `category_contract.rs`, WASM count test, clippy/fmt/doc gates green.

**Tests:**
- Unit: Jest/Vitest/Go positives; negatives.
- Integration: a fixture with tests in-scope vs excluded, asserting the bucket
  populates/empties accordingly.

**Watch For:**
- **`scope_exclude` semantics.** Per CLAUDE.md, `patterns.scope_exclude` removes
  files from the *catalog only* (they stay in the graph). The `testing` bucket is
  meaningful only when test files are included. Do **not** add test-path auto-detection
  or special-casing — that would be hidden behavior. Just document that `testing`
  populates iff test files are in the pattern scope, and let the existing config knob
  govern it.
- **Generic globals.** `test(`, `it(`, `expect(` are real words that appear outside
  tests. Anchor `^` at callee start; accept residual false positives as entropy noise.

**Seeds Forward:**
- Property-based (`fc.assert`, `hypothesis`) and snapshot-testing idioms could
  extend the regex. Playwright/Cypress E2E (`cy.`, `page.`) are arguably their own
  category. Defer.
