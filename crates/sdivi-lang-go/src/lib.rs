//! Go language adapter for sdivi-rust.
//!
//! Implements [`sdivi_parsing::adapter::LanguageAdapter`] for `.go` source files
//! using the `tree-sitter-go` grammar.
//!
//! # Thread safety
//!
//! `tree_sitter::Parser` is not `Send`. Parsers are stored in `thread_local!`
//! storage so that `GoAdapter` itself can be `Send + Sync` and participate
//! in rayon parallel parsing.

mod extract;

use std::cell::RefCell;
use std::path::Path;

use sdivi_parsing::adapter::LanguageAdapter;
use sdivi_parsing::feature_record::FeatureRecord;

use extract::{collect_hints, extract_exports, extract_imports, extract_signatures};

thread_local! {
    static PARSER: RefCell<tree_sitter::Parser> = RefCell::new({
        let mut p = tree_sitter::Parser::new();
        p.set_language(&tree_sitter_go::language())
            .expect("tree-sitter-go grammar failed to load");
        p
    });
}

/// Language adapter for Go source files.
///
/// Parses `.go` files with the `tree-sitter-go` grammar and extracts:
/// - `imports` from `import` declarations
/// - `exports` from top-level declarations with capitalized names
/// - `signatures` from function and method declarations
/// - `pattern_hints` for the patterns stage
///
/// # Examples
///
/// ```rust
/// use sdivi_lang_go::GoAdapter;
/// use sdivi_parsing::adapter::LanguageAdapter;
///
/// let adapter = GoAdapter;
/// assert_eq!(adapter.language_name(), "go");
/// assert!(adapter.file_extensions().contains(&".go"));
/// ```
pub struct GoAdapter;

impl LanguageAdapter for GoAdapter {
    fn language_name(&self) -> &'static str {
        "go"
    }

    fn file_extensions(&self) -> &[&'static str] {
        &[".go"]
    }

    /// Parses `content` and returns a [`FeatureRecord`].
    ///
    /// The tree-sitter CST is created, traversed, and **dropped** before this
    /// method returns. No tree-sitter type escapes into the returned record.
    fn parse_file(&self, path: &Path, content: String) -> FeatureRecord {
        let source = content.as_bytes();

        let (imports, exports, signatures, pattern_hints) = PARSER.with(|cell| {
            let mut parser = cell.borrow_mut();
            let tree = parser
                .parse(source, None)
                .expect("tree-sitter-go failed to parse");
            let root = tree.root_node();
            let imports = extract_imports(root, source);
            let exports = extract_exports(root, source);
            let signatures = extract_signatures(root, source);
            let hints = collect_hints(root, source);
            (imports, exports, signatures, hints)
        });

        FeatureRecord {
            path: path.to_path_buf(),
            language: "go".to_string(),
            imports,
            exports,
            signatures,
            pattern_hints,
        }
    }
}
