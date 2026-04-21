//! Pure rendering functions for ratatui widgets.
//!
//! Doctag:tui-rendering

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};

use crate::app::state::{AppState, InputMode, ResponseTab};
use crate::core::models::RequestBody;
use crate::tui::theme::Theme;

/// Draws complete UI frame.
pub fn draw(frame: &mut Frame<'_>, state: &AppState, theme: &Theme) {
    let area = frame.area();

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);

    render_header(frame, outer[0], state, theme);
    render_main(frame, outer[1], state, theme);
    render_footer(frame, outer[2], state, theme);

    if state.show_help {
        render_help_overlay(frame, area, theme);
    }

    if state.input_mode != InputMode::None {
        render_input_overlay(frame, area, state, theme);
    }

    if let Some(error) = &state.last_error {
        render_error_overlay(frame, area, error, theme);
    }
}

fn render_header(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &Theme) {
    let active_env = state
        .workspace
        .active_environment()
        .map(|env| env.name.clone())
        .unwrap_or_else(|| "none".to_string());

    let filter_label = if state.request_filter.trim().is_empty() {
        "none".to_string()
    } else {
        state.request_filter.clone()
    };

    let title = Line::from(vec![
        Span::styled("YORU", theme.title()),
        Span::styled(
            format!("  Workspace: {}", state.workspace.name),
            theme.body(),
        ),
        Span::styled(format!("  |  Env: {active_env}"), theme.body()),
        Span::styled(format!("  |  Filter: {filter_label}"), theme.muted()),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Session ")
        .style(Style::default().bg(theme.panel_bg));

    let paragraph = Paragraph::new(title).block(block).style(theme.body());
    frame.render_widget(paragraph, area);
}

fn render_main(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &Theme) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(34), Constraint::Percentage(66)])
        .split(area);

    render_navigator(frame, columns[0], state, theme);
    render_inspector(frame, columns[1], state, theme);
}

fn render_navigator(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &Theme) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(32), Constraint::Percentage(68)])
        .split(area);

    let collection_items = state
        .workspace
        .collections
        .iter()
        .enumerate()
        .map(|(idx, collection)| {
            let style = if idx == state.selected_collection_idx {
                theme.selected()
            } else {
                theme.body()
            };
            ListItem::new(collection.name.clone()).style(style)
        })
        .collect::<Vec<_>>();

    let collections = List::new(collection_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Collections ")
            .style(Style::default().bg(theme.panel_bg)),
    );
    frame.render_widget(collections, sections[0]);

    let filtered_indices = state.filtered_request_indices();
    let request_items = state
        .selected_collection()
        .map(|collection| {
            filtered_indices
                .iter()
                .map(|idx| {
                    let request = &collection.requests[*idx];
                    let text = format!("{} {}", request.method, request.name);
                    let style = if *idx == state.selected_request_idx {
                        theme.selected()
                    } else {
                        theme.body()
                    };
                    ListItem::new(text).style(style)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let request_title = format!(" Requests ({}) ", request_items.len());
    let requests = List::new(request_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(request_title)
            .style(Style::default().bg(theme.panel_bg)),
    );
    frame.render_widget(requests, sections[1]);
}

fn render_inspector(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &Theme) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(13), Constraint::Min(8)])
        .split(area);

    let request_text = if let Some(request) = state.selected_request() {
        let body_label = match &request.body {
            RequestBody::None => "none".to_string(),
            RequestBody::Raw { content, .. } => format!("raw ({} chars)", content.len()),
            RequestBody::Json { value } => format!("json ({} chars)", value.to_string().len()),
            RequestBody::FormUrlEncoded { fields } => format!("form ({} fields)", fields.len()),
        };

        vec![
            Line::from(vec![
                Span::styled("Name: ", theme.muted()),
                Span::styled(request.name.clone(), theme.body()),
                Span::styled("    Method: ", theme.muted()),
                Span::styled(request.method.to_string(), theme.body()),
            ]),
            Line::from(vec![
                Span::styled("URL: ", theme.muted()),
                Span::styled(request.url.clone(), theme.body()),
            ]),
            Line::from(vec![
                Span::styled("Auth: ", theme.muted()),
                Span::styled(auth_label(request), theme.body()),
                Span::styled("    Body: ", theme.muted()),
                Span::styled(body_label, theme.body()),
            ]),
            Line::from(vec![
                Span::styled("Headers: ", theme.muted()),
                Span::styled(request.headers.len().to_string(), theme.body()),
                Span::styled("    Query: ", theme.muted()),
                Span::styled(request.query.len().to_string(), theme.body()),
                Span::styled("    Timeout: ", theme.muted()),
                Span::styled(
                    request
                        .timeout_ms
                        .map(|value| format!("{} ms", value))
                        .unwrap_or_else(|| "default".to_string()),
                    theme.body(),
                ),
            ]),
            Line::from(vec![
                Span::styled("Editor: ", theme.muted()),
                Span::styled("i name, u url, m method, h header, p query, b body, t bearer", theme.body()),
            ]),
        ]
    } else {
        vec![Line::from(Span::styled(
            "No request selected",
            theme.muted(),
        ))]
    };

    let request_panel = Paragraph::new(request_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Request Editor ")
                .style(Style::default().bg(theme.panel_bg)),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(request_panel, sections[0]);

    let response_panel = Paragraph::new(response_lines(state, theme))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(response_title(state))
                .style(Style::default().bg(theme.panel_bg)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(response_panel, sections[1]);
}

fn response_title(state: &AppState) -> String {
    let active = match state.response_tab {
        ResponseTab::Body => "Body",
        ResponseTab::Headers => "Headers",
        ResponseTab::Logs => "Logs",
        ResponseTab::History => "History",
    };

    format!(
        " Response [{}]  Tabs: [1]Body [2]Headers [3]Logs [4]History [Tab]Next ",
        active
    )
}

fn response_lines(state: &AppState, theme: &Theme) -> Vec<Line<'static>> {
    match state.response_tab {
        ResponseTab::Body => render_response_body(state, theme),
        ResponseTab::Headers => render_response_headers(state, theme),
        ResponseTab::Logs => render_response_logs(state, theme),
        ResponseTab::History => render_history(state, theme),
    }
}

fn render_response_body(state: &AppState, theme: &Theme) -> Vec<Line<'static>> {
    let Some(response) = &state.last_response else {
        return vec![Line::from(Span::styled(
            "Run request with r/Enter to preview response body",
            theme.muted(),
        ))];
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Status: ", theme.muted()),
            Span::styled(
                format!("{} {}", response.status, response.status_text),
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("    Duration: ", theme.muted()),
            Span::styled(format!("{} ms", response.duration_ms), theme.body()),
            Span::styled("    Size: ", theme.muted()),
            Span::styled(format!("{} bytes", response.size_bytes), theme.body()),
        ]),
        Line::from(""),
    ];

    let body_preview = response.body.lines().take(26).collect::<Vec<_>>().join("\n");
    lines.push(Line::from(Span::styled(body_preview, theme.body())));
    lines
}

fn render_response_headers(state: &AppState, theme: &Theme) -> Vec<Line<'static>> {
    let Some(response) = &state.last_response else {
        return vec![Line::from(Span::styled(
            "No headers yet. Run a request first.",
            theme.muted(),
        ))];
    };

    if response.headers.is_empty() {
        return vec![Line::from(Span::styled("No response headers", theme.muted()))];
    }

    response
        .headers
        .iter()
        .take(40)
        .map(|(name, value)| {
            Line::from(vec![
                Span::styled(format!("{}: ", name), theme.muted()),
                Span::styled(value.clone(), theme.body()),
            ])
        })
        .collect()
}

fn render_response_logs(state: &AppState, theme: &Theme) -> Vec<Line<'static>> {
    let Some(response) = &state.last_response else {
        return vec![Line::from(Span::styled(
            "No script logs yet.",
            theme.muted(),
        ))];
    };

    if response.script_logs.is_empty() {
        return vec![Line::from(Span::styled(
            "No logs emitted by scripts",
            theme.muted(),
        ))];
    }

    response
        .script_logs
        .iter()
        .take(40)
        .map(|line| Line::from(Span::styled(format!("- {}", line), theme.body())))
        .collect()
}

fn render_history(state: &AppState, theme: &Theme) -> Vec<Line<'static>> {
    if state.workspace.history.is_empty() {
        return vec![Line::from(Span::styled(
            "No request history yet.",
            theme.muted(),
        ))];
    }

    state
        .workspace
        .history
        .iter()
        .rev()
        .take(32)
        .map(|entry| {
            Line::from(vec![
                Span::styled(
                    format!("{} {}", entry.method, entry.request_name),
                    theme.body(),
                ),
                Span::styled("  |  ", theme.muted()),
                Span::styled(format!("{}", entry.status), theme.body()),
                Span::styled("  |  ", theme.muted()),
                Span::styled(format!("{} ms", entry.latency_ms), theme.body()),
                Span::styled("  |  ", theme.muted()),
                Span::styled(entry.url.clone(), theme.muted()),
            ])
        })
        .collect()
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &Theme) {
    let help = Line::from(vec![
        Span::styled("Arrows", theme.muted()),
        Span::styled(" nav  ", theme.body()),
        Span::styled("r/Enter", theme.muted()),
        Span::styled(" run  ", theme.body()),
        Span::styled("n", theme.muted()),
        Span::styled(" new  ", theme.body()),
        Span::styled("d", theme.muted()),
        Span::styled(" dup  ", theme.body()),
        Span::styled("x", theme.muted()),
        Span::styled(" del  ", theme.body()),
        Span::styled("/", theme.muted()),
        Span::styled(" filter  ", theme.body()),
        Span::styled("?", theme.muted()),
        Span::styled(" help", theme.body()),
    ]);

    let message = Line::from(vec![
        Span::styled("Status: ", theme.muted()),
        Span::styled(state.status_line.clone(), theme.body()),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Controls ")
        .style(Style::default().bg(theme.panel_bg));

    let paragraph = Paragraph::new(vec![help, message])
        .block(block)
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn render_input_overlay(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &Theme) {
    let popup = centered_rect(72, 26, area);
    let title = format!(" {} ", state.input_mode.prompt());

    let content = vec![
        Line::from(Span::styled(
            "Type input and press Enter to apply",
            theme.muted(),
        )),
        Line::from(Span::styled("Esc cancels", theme.muted())),
        Line::from(""),
        Line::from(Span::styled(state.input_buffer.clone(), theme.body())),
    ];

    frame.render_widget(Clear, popup);
    let dialog = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .style(Style::default().bg(theme.panel_bg)),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(dialog, popup);
}

fn render_help_overlay(frame: &mut Frame<'_>, area: Rect, theme: &Theme) {
    let popup = centered_rect(76, 62, area);

    let content = vec![
        Line::from(Span::styled("Navigation", theme.title())),
        Line::from(Span::styled("Arrows: move collections and requests", theme.body())),
        Line::from(Span::styled("/: live filter requests", theme.body())),
        Line::from(""),
        Line::from(Span::styled("Request Actions", theme.title())),
        Line::from(Span::styled("r or Enter: run request", theme.body())),
        Line::from(Span::styled("n: new quick request", theme.body())),
        Line::from(Span::styled("d: duplicate request", theme.body())),
        Line::from(Span::styled("x: delete request", theme.body())),
        Line::from(Span::styled("m: cycle method", theme.body())),
        Line::from(Span::styled("i: edit request name", theme.body())),
        Line::from(Span::styled("u: edit URL", theme.body())),
        Line::from(Span::styled("h: add header", theme.body())),
        Line::from(Span::styled("p: add query", theme.body())),
        Line::from(Span::styled("b: edit raw body", theme.body())),
        Line::from(Span::styled("t: set bearer token", theme.body())),
        Line::from(Span::styled("e: switch environment", theme.body())),
        Line::from(""),
        Line::from(Span::styled("Response Tabs", theme.title())),
        Line::from(Span::styled("1 body, 2 headers, 3 logs, 4 history", theme.body())),
        Line::from(Span::styled("Tab: next response tab", theme.body())),
        Line::from(""),
        Line::from(Span::styled("Other", theme.title())),
        Line::from(Span::styled("c: clear error, ?: toggle help, q: quit", theme.body())),
    ];

    frame.render_widget(Clear, popup);
    let dialog = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .style(Style::default().bg(theme.panel_bg)),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(dialog, popup);
}

fn render_error_overlay(frame: &mut Frame<'_>, area: Rect, error: &str, theme: &Theme) {
    let popup = centered_rect(72, 30, area);

    let content = vec![
        Line::from(Span::styled("Last Error", Style::default().fg(theme.warning))),
        Line::from(""),
        Line::from(Span::styled(error.to_string(), theme.body())),
        Line::from(""),
        Line::from(Span::styled("Press c to clear", theme.muted())),
    ];

    frame.render_widget(Clear, popup);
    let dialog = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Error ")
                .style(Style::default().bg(theme.panel_bg)),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(dialog, popup);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(layout[1])[1]
}

fn auth_label(request: &crate::core::models::RequestTemplate) -> String {
    match &request.auth {
        crate::core::models::AuthStrategy::None => "None".to_string(),
        crate::core::models::AuthStrategy::Basic { .. } => "Basic".to_string(),
        crate::core::models::AuthStrategy::Bearer { .. } => "Bearer".to_string(),
        crate::core::models::AuthStrategy::ApiKey { in_header, .. } => {
            if *in_header {
                "API Key (header)".to_string()
            } else {
                "API Key (query)".to_string()
            }
        }
    }
}
