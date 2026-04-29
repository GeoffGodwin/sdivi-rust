//! `sdi catalog` — build and display the pattern catalog.

use std::path::Path;

use anyhow::Result;
use sdi_config::Config;
use sdi_parsing::adapter::LanguageAdapter;
use sdi_parsing::parse::parse_repository;
use sdi_patterns::build_catalog;

use crate::output;

/// Runs `sdi catalog` in `repo_root` using the given configuration.
///
/// Parses the repository, builds a [`sdi_patterns::PatternCatalog`], and
/// writes it to stdout in `format` (either `"json"` or `"text"`). Logs and
/// progress go to stderr per CLAUDE.md Rule 8.
///
/// # Errors
///
/// Returns an error if JSON serialization fails.
pub fn run(repo_root: &Path, config: &Config, format: &str) -> Result<()> {
    let adapters = all_adapters();
    eprintln!("sdi: parsing repository at {}", repo_root.display());
    let records = parse_repository(config, repo_root, &adapters);
    eprintln!("sdi: parsed {} files", records.len());

    let catalog = build_catalog(&records, &config.patterns);

    match format {
        "json" => output::json::print_catalog(&catalog)?,
        _ => output::text::print_catalog(&catalog),
    }

    Ok(())
}

/// Returns one instance of every built-in language adapter.
fn all_adapters() -> Vec<Box<dyn LanguageAdapter>> {
    vec![
        Box::new(sdi_lang_rust::RustAdapter),
        Box::new(sdi_lang_python::PythonAdapter),
        Box::new(sdi_lang_typescript::TypeScriptAdapter),
        Box::new(sdi_lang_javascript::JavaScriptAdapter),
        Box::new(sdi_lang_go::GoAdapter),
        Box::new(sdi_lang_java::JavaAdapter),
    ]
}
