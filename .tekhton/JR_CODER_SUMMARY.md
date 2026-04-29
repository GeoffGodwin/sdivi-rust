# Junior Coder Summary: Milestone 3 Blockers

**Date:** 2026-04-29
**Blockers Fixed:** 1 of 1

---

## What Was Fixed

### UTF-8 Panic in `collect_hints` — Byte-Range Indexing on Multi-Byte Characters
- **File:** `crates/sdi-lang-rust/src/extract.rs` (lines 101–108)
- **Issue:** The code performed unsafe byte-range indexing `raw[..256]` on a UTF-8 string. When a multi-byte UTF-8 character (e.g., Unicode in docstrings, identifiers, or string literals) spans the 256-byte boundary, Rust panics because string indexing requires indices to fall on valid character boundaries.
- **Fix:** Replaced with char-boundary-safe truncation using `char_indices()`:
  ```rust
  let end = raw
      .char_indices()
      .take_while(|(i, _)| *i < 256)
      .last()
      .map(|(i, c)| i + c.len_utf8())
      .unwrap_or(0);
  raw[..end].to_string()
  ```
- **How it works:** 
  1. Iterate through all (byte_index, char) pairs with `char_indices()`
  2. Keep only characters that start before byte 256
  3. Find the last such character and add its UTF-8 length to get a safe endpoint
  4. Slice the string at that boundary, preventing mid-character splits
- **Impact:** Eliminates the runtime panic; existing proptest coverage (ASCII-only `[ -~\n\t]{0,2048}`) will not catch this regression unless extended with Unicode variants.

---

## Files Modified

- `crates/sdi-lang-rust/src/extract.rs`

---

## Verification

- ✓ `cargo check -p sdi-lang-rust` — successful compilation
- ✓ Code follows Rust UTF-8 safety patterns and idioms
- ✓ Handles edge case of empty input gracefully with `unwrap_or(0)`
- ✓ String slicing now respects UTF-8 character boundaries
