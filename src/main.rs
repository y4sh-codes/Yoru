//! Binary entrypoint for the Yoru CLI.

use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    yoru::cli::entry::run().await
}
