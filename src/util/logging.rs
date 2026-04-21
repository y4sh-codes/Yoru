//! Tracing and logging bootstrap.

use tracing_subscriber::{EnvFilter, fmt};

use crate::YoruResult;

/// Initializes tracing with environment-based filtering.
pub fn init_logging() -> YoruResult<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("yoru=info"));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .try_init()
        .map_err(|err| crate::YoruError::Runtime(format!("failed to initialize logging: {err}")))?;

    Ok(())
}
