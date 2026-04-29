// simple-rust fixture: errors.rs
// Imports: 1 | Exports: 1
use std::fmt;

/// Errors produced by the simple-rust fixture library.
#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    InvalidInput(String),
    Overflow,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound(key) => write!(f, "not found: {key}"),
            AppError::InvalidInput(msg) => write!(f, "invalid input: {msg}"),
            AppError::Overflow => write!(f, "arithmetic overflow"),
        }
    }
}

impl std::error::Error for AppError {}

pub type Result<T> = std::result::Result<T, AppError>;
