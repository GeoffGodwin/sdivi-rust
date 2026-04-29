// simple-rust fixture: config.rs
// Imports: 2 | Exports: 2
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Runtime configuration for the simple-rust fixture library.
#[derive(Debug, Clone)]
pub struct Config {
    pub data_dir: PathBuf,
    pub max_records: usize,
    pub options: BTreeMap<String, String>,
}

impl Config {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Config {
            data_dir: data_dir.into(),
            max_records: 1000,
            options: BTreeMap::new(),
        }
    }

    pub fn with_option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::new(".")
    }
}

/// Loads a `Config` from a path (stub — returns default for fixture purposes).
pub fn load_config(_path: &std::path::Path) -> Config {
    Config::default()
}
