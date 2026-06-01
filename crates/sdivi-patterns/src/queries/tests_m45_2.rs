//! M45.2 unit tests for `error_handling` enrichment.
//!
//! Covers: `try_statement`, `except_clause` (Python), `catch_clause`, `throw_statement` (Java).

use super::*;

#[test]
fn try_statement_is_error_handling() {
    assert_eq!(
        category_for_node_kind("try_statement", "python"),
        Some("error_handling"),
        "try_statement must map to error_handling"
    );
}

#[test]
fn except_clause_is_error_handling() {
    assert_eq!(
        category_for_node_kind("except_clause", "python"),
        Some("error_handling"),
        "except_clause must map to error_handling (M45.2 acceptance criterion)"
    );
}

#[test]
fn catch_clause_is_error_handling() {
    assert_eq!(
        category_for_node_kind("catch_clause", "java"),
        Some("error_handling"),
        "catch_clause must map to error_handling (M45.2 acceptance criterion)"
    );
}

#[test]
fn throw_statement_is_error_handling() {
    assert_eq!(
        category_for_node_kind("throw_statement", "java"),
        Some("error_handling"),
        "throw_statement must map to error_handling (M45.2 acceptance criterion)"
    );
}
