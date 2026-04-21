//! Reqwest client factory.

use reqwest::Client;

use crate::YoruResult;

/// Builds a shared HTTP client tuned for CLI/TUI workloads.
pub fn build_http_client() -> YoruResult<Client> {
    let client = Client::builder()
        .http2_adaptive_window(true)
        .pool_idle_timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(10))
        .user_agent("yoru/0.1.0")
        .build()?;

    Ok(client)
}
