//! Yoru core library.
//!
//! Yoru is a lightweight, modular API client for terminal workflows.
//! It provides both a rich TUI experience and non-interactive CLI commands.

pub mod app;
pub mod cli;
pub mod core;
pub mod http;
pub mod storage;
pub mod tui;
pub mod util;

pub use util::error::{YoruError, YoruResult};
