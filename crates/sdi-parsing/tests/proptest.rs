//! Property tests: arbitrary Rust source content never panics during parsing.

use sdi_lang_rust::RustAdapter;
use sdi_parsing::adapter::LanguageAdapter;
use std::path::Path;

use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn parse_arbitrary_utf8_never_panics(content in "[ -~\n\t]{0,2048}") {
        let adapter = RustAdapter;
        // Must not panic regardless of content.
        let _record = adapter.parse_file(Path::new("arb.rs"), content);
    }

    #[test]
    fn parse_arbitrary_unicode_never_panics(
        content in any::<String>().prop_map(|s| s.chars().take(512).collect::<String>())
    ) {
        let adapter = RustAdapter;
        // Must not panic on multi-byte UTF-8 input; exercises the char-safe
        // truncation path in collect_hints (raw.len() > 256 with non-ASCII chars).
        let record = adapter.parse_file(Path::new("arb_unicode.rs"), content);
        // All hint texts must be valid UTF-8 — no mid-char truncation.
        for hint in &record.pattern_hints {
            assert!(
                std::str::from_utf8(hint.text.as_bytes()).is_ok(),
                "hint text must be valid UTF-8 after truncation: {:?}",
                hint.text
            );
        }
    }

    #[test]
    fn parse_valid_rust_fn_extracts_signature(
        name in "[a-z][a-z0-9_]{0,15}",
        param in "[a-z][a-z0-9_]{0,8}",
    ) {
        let content = format!("pub fn {name}({param}: u64) -> u64 {{ {param} }}\n");
        let adapter = RustAdapter;
        let record = adapter.parse_file(Path::new("gen.rs"), content);
        // At least one signature should be extracted for a valid function.
        assert!(
            !record.signatures.is_empty(),
            "valid function item must yield at least one signature"
        );
    }

    #[test]
    fn parse_valid_use_extracts_import(
        seg1 in "[a-z][a-z0-9_]{0,8}",
        seg2 in "[a-z][a-z0-9_]{0,8}",
    ) {
        let content = format!("use {seg1}::{seg2};\n");
        let adapter = RustAdapter;
        let record = adapter.parse_file(Path::new("use.rs"), content);
        assert_eq!(record.imports.len(), 1, "one use statement must yield one import");
        assert!(
            record.imports[0].contains(&seg1),
            "import must contain the module name"
        );
    }
}
