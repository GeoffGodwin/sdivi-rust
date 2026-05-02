// simple-rust fixture: models.rs
// Imports: 1 | Exports: 2
use serde::{Deserialize, Serialize};

/// A named data record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub id: u64,
    pub name: String,
    pub value: f64,
}

impl Record {
    pub fn new(id: u64, name: impl Into<String>, value: f64) -> Self {
        Record {
            id,
            name: name.into(),
            value,
        }
    }
}

/// Aggregate statistics for a collection of records.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub count: usize,
    pub total: f64,
    pub mean: f64,
}

impl Stats {
    pub fn from_records(records: &[Record]) -> Self {
        let count = records.len();
        let total: f64 = records.iter().map(|r| r.value).sum();
        let mean = if count > 0 { total / count as f64 } else { 0.0 };
        Stats { count, total, mean }
    }
}
