use sdivi_core::ExitCode;

#[test]
fn success_is_zero() {
    assert_eq!(ExitCode::Success as i32, 0);
}

#[test]
fn runtime_error_is_one() {
    assert_eq!(ExitCode::RuntimeError as i32, 1);
}

#[test]
fn config_error_is_two() {
    assert_eq!(ExitCode::ConfigError as i32, 2);
}

#[test]
fn analysis_error_is_three() {
    assert_eq!(ExitCode::AnalysisError as i32, 3);
}

#[test]
fn threshold_exceeded_is_ten() {
    assert_eq!(ExitCode::ThresholdExceeded as i32, 10);
}

#[test]
fn as_i32_matches_cast() {
    for (code, expected) in [
        (ExitCode::Success, 0i32),
        (ExitCode::RuntimeError, 1),
        (ExitCode::ConfigError, 2),
        (ExitCode::AnalysisError, 3),
        (ExitCode::ThresholdExceeded, 10),
    ] {
        assert_eq!(code.as_i32(), expected);
        assert_eq!(i32::from(code), expected);
    }
}
