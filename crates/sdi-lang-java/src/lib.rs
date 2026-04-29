//! Java language adapter for sdi-rust.
//!
//! Implements [`sdi_parsing::adapter::LanguageAdapter`] for `.java` source files
//! using the `tree-sitter-java` grammar.
//!
//! # Thread safety
//!
//! `tree_sitter::Parser` is not `Send`. Parsers are stored in `thread_local!`
//! storage so that `JavaAdapter` itself can be `Send + Sync` and participate
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
        p.set_language(&tree_sitter_java::language())
            .expect("tree-sitter-java grammar failed to load");
        p
    });
}

/// Language adapter for Java source files.
///
/// Parses `.java` files with the `tree-sitter-java` grammar and extracts:
/// - `imports` from `import` declarations
/// - `exports` from top-level `public` class and interface declarations
/// - `signatures` from `public` method declarations
/// - `pattern_hints` for the patterns stage
///
/// # Examples
///
/// ```rust
/// use sdi_lang_java::JavaAdapter;
/// use sdi_parsing::adapter::LanguageAdapter;
///
/// let adapter = JavaAdapter;
/// assert_eq!(adapter.language_name(), "java");
/// assert!(adapter.file_extensions().contains(&".java"));
/// ```
pub struct JavaAdapter;

impl LanguageAdapter for JavaAdapter {
    fn language_name(&self) -> &'static str {
        "java"
    }

    fn file_extensions(&self) -> &[&'static str] {
        &[".java"]
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
                .expect("tree-sitter-java failed to parse");
            let root = tree.root_node();
            let imports = extract_imports(root, source);
            let exports = extract_exports(root, source);
            let signatures = extract_signatures(root, source);
            let hints = collect_hints(root, source);
            (imports, exports, signatures, hints)
        });

        FeatureRecord {
            path: path.to_path_buf(),
            language: "java".to_string(),
            imports,
            exports,
            signatures,
            pattern_hints,
        }
    }
}
