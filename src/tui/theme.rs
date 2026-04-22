//! Theme palette and style system for TUI widgets.
 
use ratatui::style::{Color, Modifier, Style};
 
/// Shared style tokens.
#[derive(Debug, Clone)]
pub struct Theme {
    pub background: Color,
    pub panel_bg: Color,
    pub border: Color,
    pub primary: Color,
    pub accent: Color,
    pub warning: Color,
    pub success: Color,
    pub error_color: Color,
    pub text: Color,
    pub dim_text: Color,
    // HTTP method badge colors
    pub method_get: Color,
    pub method_post: Color,
    pub method_put: Color,
    pub method_patch: Color,
    pub method_delete: Color,
    pub method_other: Color,
    // Response status colors
    pub status_2xx: Color,
    pub status_3xx: Color,
    pub status_4xx: Color,
    pub status_5xx: Color,
}
 
impl Default for Theme {
    fn default() -> Self {
        Self {
            background:    Color::Rgb(8, 12, 18),
            panel_bg:      Color::Rgb(14, 20, 30),
            border:        Color::Rgb(35, 55, 80),
            primary:       Color::Rgb(0, 212, 255),
            accent:        Color::Rgb(255, 107, 53),
            warning:       Color::Rgb(255, 200, 0),
            success:       Color::Rgb(80, 250, 123),
            error_color:   Color::Rgb(255, 80, 80),
            text:          Color::Rgb(210, 228, 242),
            dim_text:      Color::Rgb(80, 110, 138),
            method_get:    Color::Rgb(80, 250, 123),
            method_post:   Color::Rgb(97, 175, 255),
            method_put:    Color::Rgb(255, 200, 0),
            method_patch:  Color::Rgb(255, 140, 60),
            method_delete: Color::Rgb(255, 80, 80),
            method_other:  Color::Rgb(160, 165, 180),
            status_2xx:    Color::Rgb(80, 250, 123),
            status_3xx:    Color::Rgb(97, 175, 255),
            status_4xx:    Color::Rgb(255, 200, 0),
            status_5xx:    Color::Rgb(255, 80, 80),
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
            .bg(self.primary)
            .add_modifier(Modifier::BOLD)
    }
 
    pub fn accent_style(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }
 
    pub fn key_hint(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }
 
    pub fn method_style(&self, method: &str) -> Style {
        Style::default()
            .fg(self.method_color(method))
            .add_modifier(Modifier::BOLD)
    }
 
    pub fn method_color(&self, method: &str) -> Color {
        match method {
            "GET"    => self.method_get,
            "POST"   => self.method_post,
            "PUT"    => self.method_put,
            "PATCH"  => self.method_patch,
            "DELETE" => self.method_delete,
            _        => self.method_other,
        }
    }
 
    pub fn status_style(&self, status: u16) -> Style {
        let color = match status {
            200..=299 => self.status_2xx,
            300..=399 => self.status_3xx,
            400..=499 => self.status_4xx,
            500..=599 => self.status_5xx,
            _         => self.text,
        };
        Style::default().fg(color).add_modifier(Modifier::BOLD)
    }
}
 