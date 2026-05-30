
#### Milestone 38: Pattern Category — `schema_validation`

<!-- milestone-meta
id: "38"
status: "planned"
-->

**Scope:** Add the `schema_validation` category for runtime schema / validation
declarations. TS/JS: Zod (`z.object`, `z.string`, `z.enum`), Yup
(`yup.object().shape(...)`), Valibot (`v.object`), io-ts, Superstruct. Python:
Pydantic (`BaseModel` usage, `Field(...)`, `@validator`). Detected via callee-text
on `call_expression`/`call`; depends on the M34 dispatch framework. No
parsing-layer change.

**Why this milestone exists:** Schema-as-code is now a core TS convention
(tRPC/Zod stacks, form validation, API boundary parsing). "Where and how we declare
runtime shapes" drifts meaningfully across a team, and a dedicated bucket captures
it without overlapping `type_assertions` (compile-time) or `data_access`.

**Deliverables:**
- Create `crates/sdivi-patterns/src/queries/schema_validation.rs` with per-language
  `LazyLock<Regex>` tables and `matches_callee`.
- Register in `ALL_CATEGORIES`, the M34 `CALL_DISPATCH` registry at **slot P4**
  (above `state_store`/`http_routing`/`data_access`, below `testing`/`serialization`),
  and `CATALOG_ENTRIES`.
- Update `docs/pattern-categories.md`.

**Detection (finalized — library-prefix anchored to avoid generic-method noise):**
| Language | Pattern | Examples matched | Deliberately NOT matched |
|---|---|---|---|
| TS / JS | `^(z\|yup\|v\|s)\.\w` | `z.object({...})`, `yup.string()`, `v.pipe(...)`, `s.object(...)` | bare `.string()`/`.array()` on arbitrary receivers |
| TS / JS | `\.safeParse\(` | `UserSchema.safeParse(x)` (Zod-specific name) | bare `.parse(` (collides with date/arg parsers) |
| Python | `\bField\(` , `\bconstr\(` , `\bconint\(` | `Field(default=...)`, `constr(min_length=1)` | — |

**Rationale for the tightening:** the original draft included
`\.(object\|string\|number\|...)\(` which matches any `.string()`/`.array()` call
on any receiver — far too broad. v0 anchors on the schema-library *namespace*
(`z.`/`yup.`/`v.`/`s.`) plus the Zod-unique `.safeParse(`. This reliably catches
schema *definitions* and validated parses; it intentionally misses
`SomeSchema.parse(x)` where the receiver name is arbitrary (no namespace signal in
callee text). Document that limitation — recall is traded for precision in v0.

**Migration Impact:** Additive; `list_categories()` +1. New non-zero bucket in
repos using these libraries. **Possible draw from `data_access`:** `schema.parse(x)`
matches both this and the data-access `\.(query\|read\|write\|fetch)\(`-style
regex? No — `parse` is not in the data-access set, so disjoint. Verify in the M34
disjointness corpus. `snapshot_version` stays `"1.0"`. `MIGRATION_NOTES.md` entry.

**Files to create or modify:**
- **Create:** `crates/sdivi-patterns/src/queries/schema_validation.rs`.
- **Modify:** `crates/sdivi-patterns/src/queries/mod.rs`,
  `crates/sdivi-core/src/categories.rs`.
- **Modify:** `docs/pattern-categories.md`, `MIGRATION_NOTES.md`, `CHANGELOG.md`.

**Acceptance criteria:**
- `z.object({})` → `["schema_validation"]`; `Math.max(a,b)` → `[]`.
- `category_contract.rs`, WASM count test, clippy/fmt/doc gates green.

**Tests:**
- Unit: Zod/Yup/Valibot positives; Pydantic positives; negatives.
- Disjointness corpus extended; assert no overlap with `data_access`.

**Watch For:**
- **Pydantic is class-shaped, not call-shaped.** `class U(BaseModel)` is a
  `class_definition`, already counted under `class_hierarchy`. The Python regex here
  targets the *calls* (`Field(...)`, `validator(...)`), not the class — avoid
  double-counting and document that Python coverage is partial in v0.
- **No bare-method regex.** Do not (re)introduce `\.(object\|string\|array\|...)\(`
  — it was removed precisely because it floods the bucket. Stay namespace-anchored.
  If recall on `SomeSchema.parse(x)` is later deemed essential, it requires
  receiver-type info SDIVI does not compute; treat as out of scope, not a regex tweak.

**Seeds Forward:**
- class-validator decorators (`@IsString()`) are covered by `decorators` (M36.1),
  not here — cross-reference both in the docs so the split is intentional and clear.
