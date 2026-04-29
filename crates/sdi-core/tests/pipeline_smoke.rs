//! Placeholder tracking file for Pipeline smoke tests.
//!
//! `Pipeline` will be implemented in M06 (snapshot stage assembly). Until then
//! this file verifies that the types `Pipeline` will consume — `AnalysisError`
//! and `ExitCode` — are correctly exported from `sdi_core`, and tests the
//! `AnalysisError` variants that map to the pipeline's own exit codes.

use sdi_core::{AnalysisError, ExitCode};

/// Verifies that `AnalysisError::NoGrammarsAvailable` is constructable and
/// produces a non-empty human-readable message. This is the variant Pipeline
/// will surface when no tree-sitter grammar matches any detected language
/// (Critical Rule 7 / Non-Negotiable Rule 15).
#[test]
fn no_grammars_available_is_constructable_and_has_message() {
    let err = AnalysisError::NoGrammarsAvailable;
    let msg = err.to_string();
    assert!(!msg.is_empty(), "AnalysisError display must not be empty");
}

/// Verifies that the `AnalysisError::Io` variant wraps `std::io::Error` and
/// includes its message text.
#[test]
fn io_variant_wraps_std_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "snapshot dir missing");
    let err = AnalysisError::Io(io_err);
    let msg = err.to_string();
    assert!(
        msg.contains("snapshot dir missing"),
        "AnalysisError::Io must include the underlying error message, got: {msg}"
    );
}

/// Verifies that `AnalysisError` is accessible through the `sdi_core::prelude`
/// re-export, which Pipeline embedders will use.
#[test]
fn analysis_error_accessible_via_prelude() {
    use sdi_core::prelude::AnalysisError as PreludeErr;
    let err = PreludeErr::NoGrammarsAvailable;
    assert!(!err.to_string().is_empty());
}

/// Verifies that `ExitCode::AnalysisError` (code 3) is the value `sdi snapshot`
/// would use when all detected languages lack grammars — aligning the exit-code
/// contract with Critical Rule 7.
#[test]
fn analysis_error_exit_code_is_three() {
    assert_eq!(ExitCode::AnalysisError.as_i32(), 3);
}

/// Verifies that `ExitCode::ThresholdExceeded` (code 10) is exclusively
/// reserved for `sdi check` — the only command that may emit it.
/// (Non-Negotiable Rule 9 / Critical Rule 4)
#[test]
fn threshold_exceeded_exit_code_is_ten_and_exclusive_to_check() {
    assert_eq!(ExitCode::ThresholdExceeded.as_i32(), 10);
    // No other variant may map to 10.
    let all_codes = [
        ExitCode::Success,
        ExitCode::RuntimeError,
        ExitCode::ConfigError,
        ExitCode::AnalysisError,
    ];
    for code in all_codes {
        assert_ne!(
            code.as_i32(),
            10,
            "{code:?} must not share the exit code reserved for ThresholdExceeded"
        );
    }
}
