pub mod boundaries;
pub mod catalog;
pub mod check;
pub mod diff;
pub mod init;
pub mod show;
pub mod snapshot;
pub mod trend;

use sdi_parsing::adapter::LanguageAdapter;

/// Returns one instance of every built-in language adapter.
pub(crate) fn all_adapters() -> Vec<Box<dyn LanguageAdapter>> {
    vec![
        Box::new(sdi_lang_rust::RustAdapter),
        Box::new(sdi_lang_python::PythonAdapter),
        Box::new(sdi_lang_typescript::TypeScriptAdapter),
        Box::new(sdi_lang_javascript::JavaScriptAdapter),
        Box::new(sdi_lang_go::GoAdapter),
        Box::new(sdi_lang_java::JavaAdapter),
    ]
}
