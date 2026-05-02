pub mod boundaries;
pub mod catalog;
pub mod check;
pub mod diff;
pub mod init;
pub mod show;
pub mod snapshot;
pub mod trend;

use sdivi_parsing::adapter::LanguageAdapter;

/// Returns one instance of every built-in language adapter.
pub(crate) fn all_adapters() -> Vec<Box<dyn LanguageAdapter>> {
    vec![
        Box::new(sdivi_lang_rust::RustAdapter),
        Box::new(sdivi_lang_python::PythonAdapter),
        Box::new(sdivi_lang_typescript::TypeScriptAdapter),
        Box::new(sdivi_lang_javascript::JavaScriptAdapter),
        Box::new(sdivi_lang_go::GoAdapter),
        Box::new(sdivi_lang_java::JavaAdapter),
    ]
}
