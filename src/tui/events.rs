//! Terminal event handling.

use std::time::Duration;

use crossterm::event::{self, Event, KeyEvent};

use crate::{YoruError, YoruResult};

/// App-level events used by the runtime loop.
#[derive(Debug, Clone, Copy)]
pub enum AppEvent {
    Tick,
    Input(KeyEvent),
}

/// Reads crossterm input with a fixed tick interval.
#[derive(Debug, Clone)]
pub struct EventHandler {
    tick_rate: Duration,
}

impl Default for EventHandler {
    fn default() -> Self {
        Self {
            tick_rate: Duration::from_millis(120),
        }
    }
}

impl EventHandler {
    /// Blocks until either input event or tick timeout.
    pub fn next(&self) -> YoruResult<AppEvent> {
        if event::poll(self.tick_rate)
            .map_err(|err| YoruError::Runtime(format!("event polling failed: {err}")))?
        {
            let event = event::read()
                .map_err(|err| YoruError::Runtime(format!("event read failed: {err}")))?;
            if let Event::Key(key) = event {
                return Ok(AppEvent::Input(key));
            }
        }

        Ok(AppEvent::Tick)
    }
}
