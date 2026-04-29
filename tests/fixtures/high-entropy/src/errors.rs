// high-entropy fixture: errors.rs
// Deliberately varied error-handling patterns to produce high entropy.
// Uses both try_expression (?) and match_expression for error handling.

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Parse a number, using ? for error propagation.
pub fn parse_number(s: &str) -> Result<i64> {
    let n: i64 = s.trim().parse()?;
    Ok(n)
}

/// Double a number with ? propagation.
pub fn double_parse(s: &str) -> Result<i64> {
    let n = parse_number(s)?;
    Ok(n * 2)
}

/// Triple a number with ? propagation.
pub fn triple_parse(s: &str) -> Result<i64> {
    let n = parse_number(s)?;
    Ok(n * 3)
}

/// Quad a number with ? propagation.
pub fn quad_parse(s: &str) -> Result<i64> {
    let n = parse_number(s)?;
    let m = double_parse(s)?;
    Ok(n + m)
}

/// Classify a value using match for error-aware dispatch.
pub fn classify(x: i64) -> &'static str {
    match x {
        i64::MIN..=-1 => "negative",
        0 => "zero",
        1..=100 => "small",
        _ => "large",
    }
}

/// Classify a result using match on Result variants.
pub fn classify_result(r: Result<i64>) -> &'static str {
    match r {
        Ok(n) if n > 0 => "positive",
        Ok(_) => "non-positive",
        Err(_) => "error",
    }
}

/// Another match for structural variety.
pub fn describe(x: i64) -> String {
    match x % 3 {
        0 => "divisible by 3".to_string(),
        1 => "remainder 1".to_string(),
        _ => "remainder 2".to_string(),
    }
}
