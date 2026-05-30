// simple-rust fixture: lib.rs
// Imports: 2 | Exports: 2 | Signatures: 1
// Extended in M33: added tracing macros so logging bucket is natively populated.
use std::collections::BTreeMap;
use std::fmt;

pub mod models;
pub mod utils;
pub mod errors;
pub mod config;

/// A simple counter backed by a BTreeMap.
pub struct Counter {
    counts: BTreeMap<String, u64>,
}

impl Counter {
    pub fn new() -> Self {
        tracing::info!("Counter created");
        Counter {
            counts: BTreeMap::new(),
        }
    }

    pub fn increment(&mut self, key: &str) {
        tracing::debug!("incrementing key={}", key);
        let entry = self.counts.entry(key.to_string()).or_insert(0);
        *entry += 1;
    }

    pub fn get(&self, key: &str) -> u64 {
        tracing::trace!("get key={}", key);
        *self.counts.get(key).unwrap_or(&0)
    }
}

impl fmt::Display for Counter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Counter({} keys)", self.counts.len())
    }
}
