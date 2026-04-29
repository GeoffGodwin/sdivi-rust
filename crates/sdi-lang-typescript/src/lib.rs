//! TypeScript language adapter for sdi-rust.
//!
//! Implements [`sdi_parsing::adapter::LanguageAdapter`] for `.ts` and `.tsx`
//! source files using the `tree-sitter-typescript` grammar.
//!
//! # Thread safety
//!
//! `tree_sitter::Parser` is not `Send`. Parsers are stored in `thread_local!`
//! storage so that `TypeScriptAdapter` itself can be `Send + Sync` and
//! participate in rayon parallel parsing.

mod extract;

use std::cell::RefCell;
use std::path::Path;

use sdi_parsing::adapter::LanguageAdapter;
use sdi_parsing::feature_record::FeatureRecord;

use extract::{collect_hints, extract_exports, extract_imports, extract_signatures};

thread_local! {
    static TS_PARSER: RefCell<tree_sitter::Parser> = RefCell::new({
        let mut p = tree_sitter::Parser::new();
        p.set_language(&tree_sitter_typescript::language_typescript())
            .expect("tree-sitter-typescript grammar failed to load");
        p
    });

    static TSX_PARSER: RefCell<tree_sitter::Parser> = RefCell::new({
        let mut p = tree_sitter::Parser::new();
        p.set_language(&tree_sitter_typescript::language_tsx())
            .expect("tree-sitter-typescript (TSX) grammar failed to load");
        p
    });
}

/// Language adapter for TypeScript source files (`.ts` and `.tsx`).
///
/// Parses `.ts` and `.tsx` files with the `tree-sitter-typescript` grammar and
/// extracts:
/// - `imports` from `import` statements
/// - `exports` from `export` statements at module scope
/// - `signatures` from function and method declarations
/// - `pattern_hints` for the patterns stage
///
/// # Examples
///
/// ```rust
/// use sdi_lang_typescript::TypeScriptAdapter;
/// use sdi_parsing::adapter::LanguageAdapter;
///
/// let adapter = TypeScriptAdapter;
/// assert_eq!(adapter.language_name(), "typescript");
/// assert!(adapter.file_extensions().contains(&".ts"));
/// assert!(adapter.file_extensions().contains(&".tsx"));
/// ```
pub struct TypeScriptAdapter;

impl LanguageAdapter for TypeScriptAdapter {
    fn language_name(&self) -> &'static str {
        "typescript"
    }

    fn file_extensions(&self) -> &[&'static str] {
        &[".ts", ".tsx"]
    }

    /// Parses `content` and returns a [`FeatureRecord`].
    ///
    /// The tree-sitter CST is created, traversed, and **dropped** before this
    /// method returns. No tree-sitter type escapes into the returned record.
    fn parse_file(&self, path: &Path, content: String) -> FeatureRecord {
        let source = content.as_bytes();
        let is_tsx = path.extension().map_or(false, |e| e == "tsx");

        let (imports, exports, signatures, pattern_hints) = if is_tsx {
            TSX_PARSER.with(|cell| {
                let mut parser = cell.borrow_mut();
                let tree = parser
                    .parse(source, None)
                    .expect("tree-sitter-typescript (TSX) failed to parse");
                let root = tree.root_node();
                let imports = extract_imports(root, source);
                let exports = extract_exports(root, source);
                let signatures = extract_signatures(root, source);
                let hints = collect_hints(root, source);
                (imports, exports, signatures, hints)
            })
        } else {
            TS_PARSER.with(|cell| {
                let mut parser = cell.borrow_mut();
                let tree = parser
                    .parse(source, None)
                    .expect("tree-sitter-typescript failed to parse");
                let root = tree.root_node();
                let imports = extract_imports(root, source);
                let exports = extract_exports(root, source);
                let signatures = extract_signatures(root, source);
                let hints = collect_hints(root, source);
                (imports, exports, signatures, hints)
            })
        };

        FeatureRecord {
            path: path.to_path_buf(),
            language: "typescript".to_string(),
            imports,
            exports,
            signatures,
            pattern_hints,
        }
    }
}
