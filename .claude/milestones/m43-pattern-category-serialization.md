
#### Milestone 43: Pattern Category — `serialization`

<!-- milestone-meta
id: "43"
status: "planned"
-->

**Scope:** Add the `serialization` category for (de)serialization boundaries.
TS/JS: `JSON.parse`, `JSON.stringify`, `structuredClone`. Python: `json.loads`,
`json.dumps`, `pickle.loads/dumps`. Go: `json.Marshal`, `json.Unmarshal`,
`encoding/json`-shaped calls. Callee-text on `call_expression`/`call`; depends on
M34. No parsing-layer change. Lowest-priority TS/JS-track category — included for
completeness.

**Why this milestone exists:** Serialization points are where data crosses trust /
process boundaries; their density and consistency is a modest but genuine drift
signal (e.g. drift between `JSON.parse` and a typed parser like Zod's `.parse`).
Cheap to add given the M34 framework.

**Deliverables:**
- Create `crates/sdivi-patterns/src/queries/serialization.rs` with per-language regex.
- Register in `ALL_CATEGORIES`, M34 `CALL_DISPATCH` at **slot P3 (below `testing`,
  above `schema_validation`)** — receiver-anchored and specific, so it resolves early.
  Add `CATALOG_ENTRIES`.
- Update `docs/pattern-categories.md`.

**Detection (finalized — receiver-anchored, never bare `.parse(`/`.serialize(`):**
| Language | Pattern | Examples matched |
|---|---|---|
| TS / JS | `^JSON\.(parse\|stringify)\(` , `^structuredClone\(` | `JSON.parse(s)`, `JSON.stringify(o)` |
| Python | `^(json\|pickle)\.(loads\|dumps\|load\|dump)\(` | `json.loads(s)`, `pickle.dumps(o)` |
| Go | `^json\.(Marshal\|Unmarshal\|MarshalIndent\|NewEncoder\|NewDecoder)\(` | `json.Marshal(v)`, `json.NewDecoder(r)` |

**Migration Impact:** Additive; `list_categories()` +1. New non-zero bucket;
disjoint from existing categories (none match `JSON.`/`json.`). `snapshot_version`
stays `"1.0"`. `MIGRATION_NOTES.md` entry.

**Files to create or modify:**
- **Create:** `crates/sdivi-patterns/src/queries/serialization.rs`.
- **Modify:** `crates/sdivi-patterns/src/queries/mod.rs`,
  `crates/sdivi-core/src/categories.rs`.
- **Modify:** `docs/pattern-categories.md`, `MIGRATION_NOTES.md`, `CHANGELOG.md`.

**Acceptance criteria:**
- `JSON.parse(s)` → `["serialization"]`; `json.dumps(o)` → `["serialization"]`.
- `category_contract.rs`, WASM count test, clippy/fmt/doc gates green.

**Tests:**
- Unit: TS/Python/Go positives; negatives.
- Disjointness corpus extended.

**Watch For:**
- **`json.loads` vs Python `data_access`.** The Python data-access regex is
  `^(open\(|requests\.|httpx\.|cursor\.|session\.|conn\.)` — `json.` is not in it,
  so disjoint. Confirm in the corpus.
- **Keep the set small and anchored.** Resist adding generic `.parse(`/`.serialize(`
  — they collide with schema-validation (`schema.parse`) and config parsers. Anchor
  on the `JSON`/`json`/`pickle` receivers.

**Seeds Forward:**
- Protobuf/Avro/MessagePack codecs and ORM serialize hooks are adjacent; defer until
  requested.
