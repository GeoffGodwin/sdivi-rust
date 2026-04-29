//! Rust language adapter for sdi-rust.
//!
//! Implements [`sdi_parsing::adapter::LanguageAdapter`] for `.rs` source files
//! using the `tree-sitter-rust` grammar.
//!
//! # Thread safety
//!
//! `tree_sitter::Parser` is not `Send`. Parsers are stored in `thread_local!`
//! storage so that `RustAdapter` itself can be `Send + Sync` and participate
//! in rayon parallel parsing.

mod extract;

use std::cell::RefCell;
use std::path::Path;

use sdi_parsing::adapter::LanguageAdapter;
use sdi_parsing::feature_record::FeatureRecord;

use extract::{collect_hints, extract_exports, extract_imports, extract_signatures};

thread_local! {
    static PARSER: RefCell<tree_sitter::Parser> = RefCell::new({
        let mut p = tree_sitter::Parser::new();
        p.set_language(&tree_sitter_rust::language())
            .expect("tree-sitter-rust grammar failed to load");
        p
    });
}

/// Language adapter for Rust source files.
///
/// Parses `.rs` files with the `tree-sitter-rust` grammar and extracts:
/// - `imports` from `use` declarations
/// - `exports` from public items
/// - `signatures` from function items
/// - `pattern_hints` for the patterns stage
///
/// # Examples
///
/// ```rust
/// use sdi_lang_rust::RustAdapter;
/// use sdi_parsing::adapter::LanguageAdapter;
///
/// let adapter = RustAdapter;
/// assert_eq!(adapter.language_name(), "rust");
/// assert!(adapter.file_extensions().contains(&".rs"));
/// ```
pub struct RustAdapter;

impl LanguageAdapter for RustAdapter {
    fn language_name(&self) -> &'static str {
        "rust"
    }

    fn file_extensions(&self) -> &[&'static str] {
        &[".rs"]
    }

    /// Parses `content` and returns a [`FeatureRecord`].
    ///
    /// The tree-sitter CST is created, traversed, and **dropped** before this
    /// method returns. No tree-sitter type escapes into the returned record.
    fn parse_file(&self, path: &Path, content: String) -> FeatureRecord {
        #[cfg(feature = "test-tree-counter")]
        sdi_parsing::ACTIVE_TREES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let source = content.as_bytes();

        let (imports, exports, signatures, pattern_hints) = PARSER.with(|cell| {
            let mut parser = cell.borrow_mut();
            let tree = parser
                .parse(source, None)
                .expect("tree-sitter-rust failed to parse");

            let root = tree.root_node();
            let imports = extract_imports(root, source);
            let exports = extract_exports(root, source);
            let signatures = extract_signatures(root, source);
            let hints = collect_hints(root, source);
            // tree is dropped here — CST does not escape this closure.
            (imports, exports, signatures, hints)
        });

        #[cfg(feature = "test-tree-counter")]
        sdi_parsing::ACTIVE_TREES.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);

        FeatureRecord {
            path: path.to_path_buf(),
            language: "rust".to_string(),
            imports,
            exports,
            signatures,
            pattern_hints,
        }
    }
}
