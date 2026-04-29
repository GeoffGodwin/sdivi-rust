use tracing_subscriber::{EnvFilter, fmt};

/// Initialise the `tracing` subscriber that writes structured logs to stderr.
///
/// Log level is read from `SDI_LOG_LEVEL` (default: `warn`).
pub fn init() {
    let filter = EnvFilter::try_from_env("SDI_LOG_LEVEL")
        .unwrap_or_else(|_| EnvFilter::new("warn"));

    fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();
}
