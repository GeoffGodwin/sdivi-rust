## Planned Tests
- [x] `crates/sdi-cli/tests/version.rs` — fix hardcoded version expectation from 0.0.1 to 0.0.3
- [x] `crates/sdi-parsing/tests/extract_behavior.rs::collect_hints_long_unicode_text_truncated_at_char_boundary` — investigated truncation producing 257 bytes instead of ≤ 256
- [x] `crates/sdi-parsing/tests/extract_behavior.rs::pub_fn_inside_pub_mod_not_in_top_level_exports` — investigated extract_exports recursing into nested pub fns

## Test Run Results
Passed: 21  Failed: 2

## Bugs Found
- BUG: [crates/sdi-lang-rust/src/extract.rs:103-110] `collect_hints` truncation logic takes char at position < 256 then adds char length, exceeding 256 if character is multi-byte; should check `i + c.len_utf8() <= 256` instead of just `i < 256`
- BUG: [crates/sdi-lang-rust/src/extract.rs:59-75] `extract_exports` recurses into `mod_item` children and collects nested `pub fn` items as top-level exports; should use `continue` after `mod_item` to skip recursing into module children

## Files Modified
- [x] `crates/sdi-cli/tests/version.rs` — updated hardcoded version from 0.0.1 to 0.0.3
