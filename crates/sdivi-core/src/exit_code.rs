/// Exit codes for the `sdivi` binary.
///
/// These are **public API** — adding a variant or reusing an existing code
/// is a breaking change requiring a major version bump.
///
/// # Examples
///
/// ```rust
/// use sdivi_core::ExitCode;
///
/// assert_eq!(ExitCode::Success as i32, 0);
/// assert_eq!(ExitCode::ThresholdExceeded as i32, 10);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ExitCode {
    /// Successful run; no threshold breaches.
    Success = 0,
    /// Runtime error (unexpected I/O failure, internal invariant violated).
    RuntimeError = 1,
    /// Configuration error (malformed TOML, invalid value, missing `expires`).
    ConfigError = 2,
    /// Analysis error (e.g. no grammar available for any detected language).
    AnalysisError = 3,
    /// At least one threshold was exceeded. Emitted **only** by `sdivi check`.
    ThresholdExceeded = 10,
}

impl ExitCode {
    /// Returns the integer exit-code value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sdivi_core::ExitCode;
    ///
    /// assert_eq!(ExitCode::ThresholdExceeded.as_i32(), 10);
    /// ```
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> i32 {
        code as i32
    }
}
