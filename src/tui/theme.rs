//! Theme palette and style system for TUI widgets.

use ratatui::style::{Color, Modifier, Style};

/// Shared style tokens.
#[derive(Debug, Clone)]
pub struct Theme {
    pub background: Color,
    pub panel_bg: Color,
    pub primary: Color,
    pub accent: Color,
    pub warning: Color,
    pub success: Color,
    pub text: Color,
    pub dim_text: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: Color::Rgb(16, 20, 26),
            panel_bg: Color::Rgb(28, 35, 44),
            primary: Color::Rgb(66, 184, 131),
            accent: Color::Rgb(245, 180, 51),
            warning: Color::Rgb(239, 83, 80),
            success: Color::Rgb(139, 195, 74),
            text: Color::Rgb(232, 236, 240),
            dim_text: Color::Rgb(144, 156, 169),
        }
    }
}

impl Theme {
    pub fn title(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    pub fn body(&self) -> Style {
        Style::default().fg(self.text)
    }

    pub fn muted(&self) -> Style {
        Style::default().fg(self.dim_text)
    }

    pub fn selected(&self) -> Style {
        Style::default()
            .fg(self.background)
            .bg(self.accent)
            .add_modifier(Modifier::BOLD)
    }
}
