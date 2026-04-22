//! Pure rendering functions for ratatui widgets.
//!
//! Doctag:tui-rendering

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};

use crate::app::state::{
    AppState, ConfirmAction, InputMode, ResponseTab, Screen, SplashInputMode,
};
use crate::core::models::RequestBody;
use crate::storage::fs_store::WorkspaceEntry;
use crate::tui::theme::Theme;

// ─── Entry point ─────────────────────────────────────────────────────────────

pub fn draw(frame: &mut Frame<'_>, state: &AppState, theme: &Theme) {
    let area = frame.area();

    if state.screen == Screen::Splash {
        render_splash(frame, area, state, theme);
        return;
    }

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(5),
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

// ─── Splash / Workspace Picker ────────────────────────────────────────────────

fn render_splash(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &Theme) {
    // Solid background
    frame.render_widget(
        Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(theme.background)),
        area,
    );

    let vl = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),   // top gap
            Constraint::Length(7),   // logo
            Constraint::Length(1),   // subtitle
            Constraint::Length(1),   // gap
            Constraint::Min(6),      // workspace picker (stretches)
            Constraint::Length(1),   // gap
            Constraint::Length(1),   // hints
            Constraint::Length(1),   // bottom gap
        ])
        .split(area);

    // ── Logo ──────────────────────────────────────────────────────────────────
    let logo = vec![
        Line::from(Span::styled(
            "██╗   ██╗ ██████╗ ██████╗ ██╗   ██╗",
            theme.title(),
        )),
        Line::from(Span::styled(
            "╚██╗ ██╔╝██╔═══██╗██╔══██╗██║   ██║",
            theme.title(),
        )),
        Line::from(Span::styled(
            " ╚████╔╝ ██║   ██║██████╔╝██║   ██║",
            theme.title(),
        )),
        Line::from(Span::styled(
            "  ╚██╔╝  ██║   ██║██╔══██╗██║   ██║",
            theme.title(),
        )),
        Line::from(Span::styled(
            "   ██║    ╚██████╔╝██║  ██║╚██████╔╝",
            theme.title(),
        )),
        Line::from(Span::styled(
            "   ╚═╝     ╚═════╝ ╚═╝  ╚═╝ ╚═════╝",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::DIM),
        )),
    ];
    frame.render_widget(
        Paragraph::new(logo).alignment(Alignment::Center),
        vl[1],
    );

    // Subtitle
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Terminal API Client  ·  ", theme.muted()),
            Span::styled("Postman for the shell", Style::default().fg(theme.accent)),
        ]))
        .alignment(Alignment::Center),
        vl[2],
    );

    // ── Workspace picker centred box ──────────────────────────────────────────
    let picker_hl = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .split(vl[4]);

    render_workspace_picker(frame, picker_hl[1], state, theme);

    // ── Overlay: confirm delete ───────────────────────────────────────────────
    if let Some(ConfirmAction::DeleteWorkspace(slug)) = &state.splash_confirm {
        render_confirm_delete_overlay(frame, area, slug, theme);
    }

    // ── Overlay: splash input (new / rename) ──────────────────────────────────
    if state.splash_input_mode != SplashInputMode::None {
        render_splash_input_overlay(frame, area, state, theme);
    }

    // ── Hints bar ────────────────────────────────────────────────────────────
    let kh = theme.key_hint();
    let sep = Style::default().fg(theme.border);
    let hints = Line::from(vec![
        Span::styled("↑↓", kh),
        Span::styled(" navigate  ", theme.muted()),
        Span::styled("│", sep),
        Span::styled(" Enter", kh),
        Span::styled(" open  ", theme.muted()),
        Span::styled("│", sep),
        Span::styled(" n", kh),
        Span::styled(" new  ", theme.muted()),
        Span::styled("│", sep),
        Span::styled(" r", kh),
        Span::styled(" rename  ", theme.muted()),
        Span::styled("│", sep),
        Span::styled(" x", kh),
        Span::styled(" delete  ", theme.muted()),
        Span::styled("│", sep),
        Span::styled(" ?", kh),
        Span::styled(" help  ", theme.muted()),
        Span::styled("│", sep),
        Span::styled(" q", kh),
        Span::styled(" quit", theme.muted()),
    ]);
    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        vl[6],
    );
}

fn render_workspace_picker(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &AppState,
    theme: &Theme,
) {
    // Split into list (left 55%) and detail pane (right 45%)
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    // ── Workspace list ────────────────────────────────────────────────────────
    let items: Vec<ListItem> = state
        .available_workspaces
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let is_active = entry.slug == state.active_slug;
            let is_sel    = idx == state.splash_selected_idx;

            let active_badge = if is_active {
                Span::styled(" ● ", Style::default().fg(theme.success).add_modifier(Modifier::BOLD))
            } else {
                Span::styled("   ", theme.muted())
            };

            let name_span = if is_sel {
                Span::styled(
                    format!("{:<28}", truncate(&entry.display_name, 27)),
                    Style::default()
                        .fg(theme.background)
                        .bg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    format!("{:<28}", truncate(&entry.display_name, 27)),
                    theme.body(),
                )
            };

            let meta = Span::styled(
                format!(
                    " {}c  {}r",
                    entry.collections,
                    entry.requests,
                ),
                theme.muted(),
            );

            ListItem::new(Line::from(vec![active_badge, name_span, meta]))
        })
        .collect();

    let list_title = format!(" Workspaces ({}) ", state.available_workspaces.len());
    frame.render_widget(
        List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(list_title)
                .title_style(theme.muted())
                .style(Style::default().bg(theme.panel_bg)),
        ),
        cols[0],
    );

    // ── Detail pane ───────────────────────────────────────────────────────────
    let detail_lines = if let Some(entry) = state.splash_selected_entry() {
        let is_active = entry.slug == state.active_slug;
        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Name         ", theme.muted()),
                Span::styled(entry.display_name.clone(), theme.body()),
            ]),
            Line::from(vec![
                Span::styled("  Slug         ", theme.muted()),
                Span::styled(entry.slug.clone(), theme.muted()),
            ]),
            Line::from(vec![
                Span::styled("  Collections  ", theme.muted()),
                Span::styled(entry.collections.to_string(), theme.body()),
            ]),
            Line::from(vec![
                Span::styled("  Requests     ", theme.muted()),
                Span::styled(entry.requests.to_string(), theme.body()),
            ]),
            Line::from(vec![
                Span::styled("  Environments ", theme.muted()),
                Span::styled(entry.environments.to_string(), theme.body()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Status       ", theme.muted()),
                if is_active {
                    Span::styled("● active", Style::default().fg(theme.success).add_modifier(Modifier::BOLD))
                } else {
                    Span::styled("○ inactive", theme.muted())
                },
            ]),
            Line::from(""),
            if is_active {
                Line::from(Span::styled(
                    "  [ Press Enter to re-open ]",
                    Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(Span::styled(
                    "  [ Press Enter to open ]",
                    Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
                ))
            },
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No workspaces yet.",
                theme.muted(),
            )),
            Line::from(Span::styled(
                "  Press  n  to create one.",
                theme.muted(),
            )),
        ]
    };

    frame.render_widget(
        Paragraph::new(detail_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(" Details ")
                .title_style(theme.muted())
                .style(Style::default().bg(theme.panel_bg)),
        ),
        cols[1],
    );
}

fn render_confirm_delete_overlay(
    frame: &mut Frame<'_>,
    area: Rect,
    slug: &str,
    theme: &Theme,
) {
    let popup = centered_rect(56, 26, area);

    let content = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Delete this workspace?",
            Style::default()
                .fg(theme.error_color)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Workspace: ", theme.muted()),
            Span::styled(slug.to_string(), theme.body()),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  This cannot be undone.",
            Style::default().fg(theme.warning),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", theme.muted()),
            Span::styled("y", theme.key_hint()),
            Span::styled(" / ", theme.muted()),
            Span::styled("Enter", theme.key_hint()),
            Span::styled("  confirm  ·  ", theme.muted()),
            Span::styled("n", theme.key_hint()),
            Span::styled(" / ", theme.muted()),
            Span::styled("Esc", theme.key_hint()),
            Span::styled("  cancel", theme.muted()),
        ]),
    ];

    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.error_color))
                    .title(" Confirm Delete ")
                    .title_style(
                        Style::default()
                            .fg(theme.error_color)
                            .add_modifier(Modifier::BOLD),
                    )
                    .style(Style::default().bg(theme.panel_bg)),
            )
            .wrap(Wrap { trim: true }),
        popup,
    );
}

fn render_splash_input_overlay(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &AppState,
    theme: &Theme,
) {
    let popup = centered_rect(60, 20, area);
    let title = format!(" {} ", state.splash_input_mode.prompt());

    let content = vec![
        Line::from(Span::styled(
            "Type and press Enter to confirm  ·  Esc to cancel",
            theme.muted(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("› ", theme.key_hint()),
            Span::styled(state.splash_input_buffer.clone(), theme.body()),
            Span::styled("█", Style::default().fg(theme.primary)),
        ]),
    ];

    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.primary))
                    .title(title)
                    .title_style(
                        Style::default()
                            .fg(theme.primary)
                            .add_modifier(Modifier::BOLD),
                    )
                    .style(Style::default().bg(theme.panel_bg)),
            )
            .wrap(Wrap { trim: true }),
        popup,
    );
}

// ─── Header bar ──────────────────────────────────────────────────────────────

fn render_header(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &Theme) {
    let active_env = state
        .workspace
        .active_environment()
        .map(|e| e.name.clone())
        .unwrap_or_else(|| "none".to_string());

    let total_requests: usize = state
        .workspace
        .collections
        .iter()
        .map(|c| c.requests.len())
        .sum();

    let filter_label = if state.request_filter.trim().is_empty() {
        String::new()
    } else {
        format!("  ⌕ {}", state.request_filter)
    };

    let title = Line::from(vec![
        Span::styled(
            " YORU ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("│", Style::default().fg(theme.border)),
        Span::styled(format!("  {}  ", state.workspace.name), theme.body()),
        Span::styled("│", Style::default().fg(theme.border)),
        Span::styled(
            format!(
                "  {} col  ·  {} req  ",
                state.workspace.collections.len(),
                total_requests
            ),
            theme.muted(),
        ),
        Span::styled("│", Style::default().fg(theme.border)),
        Span::styled(
            format!("  env: {}  ", active_env),
            Style::default().fg(theme.success),
        ),
        Span::styled(
            filter_label,
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::ITALIC),
        ),
    ]);

    frame.render_widget(
        Paragraph::new(title).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.panel_bg)),
        ),
        area,
    );
}

// ─── Main area ───────────────────────────────────────────────────────────────

fn render_main(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &Theme) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(32), Constraint::Percentage(68)])
        .split(area);

    render_navigator(frame, cols[0], state, theme);
    render_inspector(frame, cols[1], state, theme);
}

fn render_navigator(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &Theme) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(28), Constraint::Percentage(72)])
        .split(area);

    let col_items: Vec<ListItem> = state
        .workspace
        .collections
        .iter()
        .enumerate()
        .map(|(idx, col)| {
            let label = format!("  {}  ({})", col.name, col.requests.len());
            let style = if idx == state.selected_collection_idx {
                theme.selected()
            } else {
                theme.body()
            };
            ListItem::new(label).style(style)
        })
        .collect();

    frame.render_widget(
        List::new(col_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(" Collections ")
                .title_style(theme.muted())
                .style(Style::default().bg(theme.panel_bg)),
        ),
        sections[0],
    );

    let filtered = state.filtered_request_indices();
    let req_items: Vec<ListItem> = state
        .selected_collection()
        .map(|col| {
            filtered
                .iter()
                .map(|idx| {
                    let req = &col.requests[*idx];
                    let method = req.method.to_string();
                    let badge  = format!(" {:<7}", method);
                    let name   = format!(" {}", req.name);

                    if *idx == state.selected_request_idx {
                        ListItem::new(Line::from(vec![
                            Span::styled(
                                badge,
                                Style::default()
                                    .fg(theme.background)
                                    .bg(theme.method_color(&method))
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(name, theme.selected()),
                        ]))
                    } else {
                        ListItem::new(Line::from(vec![
                            Span::styled(badge, theme.method_style(&method)),
                            Span::styled(name, theme.body()),
                        ]))
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let req_title = format!(" Requests ({}) ", req_items.len());
    frame.render_widget(
        List::new(req_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(req_title)
                .title_style(theme.muted())
                .style(Style::default().bg(theme.panel_bg)),
        ),
        sections[1],
    );
}

fn render_inspector(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &Theme) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(13), Constraint::Min(8)])
        .split(area);

    let request_text = if let Some(req) = state.selected_request() {
        let body_label = match &req.body {
            RequestBody::None                       => "none".to_string(),
            RequestBody::Raw { content, .. }        => format!("raw ({} chars)", content.len()),
            RequestBody::Json { value }             => format!("json ({} chars)", value.to_string().len()),
            RequestBody::FormUrlEncoded { fields }  => format!("form ({} fields)", fields.len()),
        };

        let scripts = match (req.pre_request_script.is_some(), req.test_script.is_some()) {
            (true, true)   => "pre + test",
            (true, false)  => "pre",
            (false, true)  => "test",
            (false, false) => "none",
        };

        let tags = if req.tags.is_empty() { "—".to_string() } else { req.tags.join(", ") };
        let method = req.method.to_string();

        vec![
            Line::from(vec![
                Span::styled(" Name    ", theme.muted()),
                Span::styled(req.name.clone(), theme.body()),
            ]),
            Line::from(vec![
                Span::styled(" Method  ", theme.muted()),
                Span::styled(method.clone(), theme.method_style(&method)),
                Span::styled("   Auth    ", theme.muted()),
                Span::styled(auth_label(req), theme.body()),
            ]),
            Line::from(vec![
                Span::styled(" URL     ", theme.muted()),
                Span::styled(req.url.clone(), theme.body()),
            ]),
            Line::from(vec![
                Span::styled(" Headers ", theme.muted()),
                Span::styled(req.headers.len().to_string(), theme.body()),
                Span::styled("   Query   ", theme.muted()),
                Span::styled(req.query.len().to_string(), theme.body()),
                Span::styled("   Body    ", theme.muted()),
                Span::styled(body_label, theme.body()),
            ]),
            Line::from(vec![
                Span::styled(" Timeout ", theme.muted()),
                Span::styled(
                    req.timeout_ms.map(|v| format!("{}ms", v)).unwrap_or_else(|| "default".to_string()),
                    theme.body(),
                ),
                Span::styled("   Scripts ", theme.muted()),
                Span::styled(scripts.to_string(), theme.body()),
                Span::styled("   Tags    ", theme.muted()),
                Span::styled(tags, theme.muted()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" Edit  ", theme.muted()),
                Span::styled("i", theme.key_hint()), Span::styled(" name  ", theme.muted()),
                Span::styled("u", theme.key_hint()), Span::styled(" url  ", theme.muted()),
                Span::styled("m", theme.key_hint()), Span::styled(" method  ", theme.muted()),
                Span::styled("b", theme.key_hint()), Span::styled(" body  ", theme.muted()),
                Span::styled("h", theme.key_hint()), Span::styled(" header  ", theme.muted()),
                Span::styled("p", theme.key_hint()), Span::styled(" query", theme.muted()),
            ]),
            Line::from(vec![
                Span::styled(" Auth  ", theme.muted()),
                Span::styled("t", theme.key_hint()), Span::styled(" bearer  ", theme.muted()),
                Span::styled("a", theme.key_hint()), Span::styled(" basic  ", theme.muted()),
                Span::styled("k", theme.key_hint()), Span::styled(" api-key  ", theme.muted()),
                Span::styled("T", theme.key_hint()), Span::styled(" timeout", theme.muted()),
            ]),
        ]
    } else {
        vec![Line::from(Span::styled(
            " No request selected — press n to add one",
            theme.muted(),
        ))]
    };

    frame.render_widget(
        Paragraph::new(request_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(" Request Inspector ")
                    .title_style(theme.muted())
                    .style(Style::default().bg(theme.panel_bg)),
            )
            .wrap(Wrap { trim: true }),
        sections[0],
    );

    frame.render_widget(
        Paragraph::new(response_lines(state, theme))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(response_title(state))
                    .title_style(theme.muted())
                    .style(Style::default().bg(theme.panel_bg)),
            )
            .wrap(Wrap { trim: false })
            .scroll((state.response_scroll, 0)),
        sections[1],
    );
}

// ─── Response helpers ─────────────────────────────────────────────────────────

fn response_title(state: &AppState) -> String {
    let active = match state.response_tab {
        ResponseTab::Body    => "Body",
        ResponseTab::Headers => "Headers",
        ResponseTab::Logs    => "Logs",
        ResponseTab::History => "History",
    };
    let scroll = if state.response_scroll > 0 {
        format!(" ↕{}", state.response_scroll)
    } else {
        String::new()
    };
    format!(
        " Response · [1]Body [2]Headers [3]Logs [4]History  active:{}{} ",
        active, scroll
    )
}

fn response_lines(state: &AppState, theme: &Theme) -> Vec<Line<'static>> {
    match state.response_tab {
        ResponseTab::Body    => render_response_body(state, theme),
        ResponseTab::Headers => render_response_headers(state, theme),
        ResponseTab::Logs    => render_response_logs(state, theme),
        ResponseTab::History => render_history(state, theme),
    }
}

fn render_response_body(state: &AppState, theme: &Theme) -> Vec<Line<'static>> {
    let Some(resp) = &state.last_response else {
        return vec![
            Line::from(""),
            Line::from(Span::styled("  No response yet.", theme.muted())),
            Line::from(Span::styled(
                "  Select a request and press  r  or  Enter  to run it.",
                theme.muted(),
            )),
        ];
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled("  Status   ", theme.muted()),
            Span::styled(format!("{} {}", resp.status, resp.status_text), theme.status_style(resp.status)),
            Span::styled("   Duration  ", theme.muted()),
            Span::styled(format!("{} ms", resp.duration_ms), theme.body()),
            Span::styled("   Size  ", theme.muted()),
            Span::styled(format_size(resp.size_bytes), theme.body()),
        ]),
        Line::from(Span::styled(
            "─".repeat(60),
            Style::default().fg(theme.border),
        )),
    ];

    for line in resp.body.lines() {
        lines.push(Line::from(Span::styled(line.to_string(), theme.body())));
    }

    lines
}

fn render_response_headers(state: &AppState, theme: &Theme) -> Vec<Line<'static>> {
    let Some(resp) = &state.last_response else {
        return vec![Line::from(Span::styled("  No headers yet — run a request first.", theme.muted()))];
    };

    if resp.headers.is_empty() {
        return vec![Line::from(Span::styled("  No response headers.", theme.muted()))];
    }

    let mut lines = vec![
        Line::from(Span::styled(
            format!("  {} headers", resp.headers.len()),
            theme.muted(),
        )),
        Line::from(Span::styled("─".repeat(60), Style::default().fg(theme.border))),
    ];

    for (name, value) in resp.headers.iter().take(60) {
        lines.push(Line::from(vec![
            Span::styled(format!("  {}: ", name), theme.muted()),
            Span::styled(value.clone(), theme.body()),
        ]));
    }
    lines
}

fn render_response_logs(state: &AppState, theme: &Theme) -> Vec<Line<'static>> {
    let Some(resp) = &state.last_response else {
        return vec![Line::from(Span::styled("  No script logs yet.", theme.muted()))];
    };
    if resp.script_logs.is_empty() {
        return vec![Line::from(Span::styled("  Scripts ran with no log() calls.", theme.muted()))];
    }
    resp.script_logs
        .iter()
        .map(|l| Line::from(Span::styled(format!("  › {}", l), theme.body())))
        .collect()
}

fn render_history(state: &AppState, theme: &Theme) -> Vec<Line<'static>> {
    if state.workspace.history.is_empty() {
        return vec![Line::from(Span::styled("  No history yet — run some requests.", theme.muted()))];
    }

    let mut lines = vec![
        Line::from(Span::styled(
            format!("  {} entries (newest first)", state.workspace.history.len()),
            theme.muted(),
        )),
        Line::from(Span::styled("─".repeat(60), Style::default().fg(theme.border))),
    ];

    for entry in state.workspace.history.iter().rev().take(60) {
        let method = entry.method.to_string();
        lines.push(Line::from(vec![
            Span::styled(format!(" {:<7}", method), theme.method_style(&method)),
            Span::styled(format!("{:<20}", entry.request_name), theme.body()),
            Span::styled(format!("  {}  ", entry.status), theme.status_style(entry.status)),
            Span::styled(format!("{} ms  ", entry.latency_ms), theme.muted()),
            Span::styled(entry.url.clone(), theme.muted()),
        ]));
    }
    lines
}

// ─── Footer ──────────────────────────────────────────────────────────────────

fn render_footer(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &Theme) {
    let kh  = theme.key_hint();
    let sep = Style::default().fg(theme.border);

    let nav_line = Line::from(vec![
        Span::styled("↑↓←→", kh), Span::styled(" navigate  ", theme.muted()),
        Span::styled("│", sep),
        Span::styled(" r", kh), Span::styled("/Enter", kh), Span::styled(" run  ", theme.muted()),
        Span::styled("│", sep),
        Span::styled(" n", kh), Span::styled(" new req  ", theme.muted()),
        Span::styled("│", sep),
        Span::styled(" N", kh), Span::styled(" new col  ", theme.muted()),
        Span::styled("│", sep),
        Span::styled(" C", kh), Span::styled(" rename col  ", theme.muted()),
        Span::styled("│", sep),
        Span::styled(" d", kh), Span::styled(" dup  ", theme.muted()),
        Span::styled("│", sep),
        Span::styled(" x", kh), Span::styled(" del  ", theme.muted()),
        Span::styled("│", sep),
        Span::styled(" /", kh), Span::styled(" filter  ", theme.muted()),
        Span::styled("│", sep),
        Span::styled(" W", kh), Span::styled(" workspaces  ", theme.muted()),
        Span::styled("│", sep),
        Span::styled(" ?", kh), Span::styled(" help  ", theme.muted()),
        Span::styled("│", sep),
        Span::styled(" q", kh), Span::styled(" quit", theme.muted()),
    ]);

    let auth_line = Line::from(vec![
        Span::styled("Auth  ", theme.muted()),
        Span::styled("t", kh), Span::styled(" bearer  ", theme.muted()),
        Span::styled("│", sep),
        Span::styled(" a", kh), Span::styled(" basic  ", theme.muted()),
        Span::styled("│", sep),
        Span::styled(" k", kh), Span::styled(" api-key  ", theme.muted()),
        Span::styled("│", sep),
        Span::styled("Edit  ", theme.muted()),
        Span::styled("i", kh), Span::styled(" name  ", theme.muted()),
        Span::styled("│", sep),
        Span::styled(" u", kh), Span::styled(" url  ", theme.muted()),
        Span::styled("│", sep),
        Span::styled(" m", kh), Span::styled(" method  ", theme.muted()),
        Span::styled("│", sep),
        Span::styled(" b", kh), Span::styled(" body  ", theme.muted()),
        Span::styled("│", sep),
        Span::styled(" T", kh), Span::styled(" timeout  ", theme.muted()),
        Span::styled("│", sep),
        Span::styled(" e", kh), Span::styled(" env", theme.muted()),
    ]);

    let status_style = if state.last_error.is_some() {
        Style::default().fg(theme.error_color)
    } else {
        Style::default().fg(theme.success)
    };

    let status_line = Line::from(vec![
        Span::styled("● ", status_style),
        Span::styled(state.status_line.clone(), theme.body()),
    ]);

    frame.render_widget(
        Paragraph::new(vec![nav_line, auth_line, status_line])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.panel_bg)),
            )
            .wrap(Wrap { trim: true }),
        area,
    );
}

// ─── Overlays ────────────────────────────────────────────────────────────────

fn render_input_overlay(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &Theme) {
    let popup = centered_rect(72, 22, area);
    let title = format!(" {} ", state.input_mode.prompt());

    let content = vec![
        Line::from(Span::styled(
            "Type and press Enter to confirm  ·  Esc to cancel",
            theme.muted(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("› ", theme.key_hint()),
            Span::styled(state.input_buffer.clone(), theme.body()),
            Span::styled("█", Style::default().fg(theme.primary)),
        ]),
    ];

    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.primary))
                    .title(title)
                    .title_style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
                    .style(Style::default().bg(theme.panel_bg)),
            )
            .wrap(Wrap { trim: true }),
        popup,
    );
}

fn render_help_overlay(frame: &mut Frame<'_>, area: Rect, theme: &Theme) {
    let popup = centered_rect(78, 84, area);
    let kh = theme.key_hint();

    let content = vec![
        Line::from(vec![
            Span::styled("  YORU Help  ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled("                            Press  Esc  or  ?  to close", theme.muted()),
        ]),
        Line::from(Span::styled("─".repeat(70), Style::default().fg(theme.border))),
        Line::from(""),
        Line::from(Span::styled("  Navigation", theme.title())),
        Line::from(vec![Span::styled("  ↑ / ↓ ", kh), Span::styled("  move between requests", theme.muted())]),
        Line::from(vec![Span::styled("  ← / → ", kh), Span::styled("  switch collection", theme.muted())]),
        Line::from(vec![Span::styled("  /     ", kh), Span::styled("  live-filter requests", theme.muted())]),
        Line::from(""),
        Line::from(Span::styled("  Request Actions", theme.title())),
        Line::from(vec![Span::styled("  r  Enter ", kh), Span::styled("  run selected request", theme.muted())]),
        Line::from(vec![Span::styled("  n        ", kh), Span::styled("  add quick request", theme.muted())]),
        Line::from(vec![Span::styled("  d        ", kh), Span::styled("  duplicate request", theme.muted())]),
        Line::from(vec![Span::styled("  x        ", kh), Span::styled("  delete request", theme.muted())]),
        Line::from(vec![Span::styled("  m        ", kh), Span::styled("  cycle HTTP method", theme.muted())]),
        Line::from(""),
        Line::from(Span::styled("  Request Editing", theme.title())),
        Line::from(vec![Span::styled("  i        ", kh), Span::styled("  edit request name", theme.muted())]),
        Line::from(vec![Span::styled("  u        ", kh), Span::styled("  edit URL", theme.muted())]),
        Line::from(vec![Span::styled("  h        ", kh), Span::styled("  add header  Key:Value", theme.muted())]),
        Line::from(vec![Span::styled("  p        ", kh), Span::styled("  add query param  key=value", theme.muted())]),
        Line::from(vec![Span::styled("  b        ", kh), Span::styled("  edit raw request body", theme.muted())]),
        Line::from(vec![Span::styled("  T        ", kh), Span::styled("  set timeout in ms", theme.muted())]),
        Line::from(""),
        Line::from(Span::styled("  Authentication", theme.title())),
        Line::from(vec![Span::styled("  t        ", kh), Span::styled("  bearer token", theme.muted())]),
        Line::from(vec![Span::styled("  a        ", kh), Span::styled("  basic auth  username:password", theme.muted())]),
        Line::from(vec![Span::styled("  k        ", kh), Span::styled("  api key  name:value:h/q", theme.muted())]),
        Line::from(""),
        Line::from(Span::styled("  Collections & Workspaces", theme.title())),
        Line::from(vec![Span::styled("  N        ", kh), Span::styled("  new collection", theme.muted())]),
        Line::from(vec![Span::styled("  C        ", kh), Span::styled("  rename collection", theme.muted())]),
        Line::from(vec![Span::styled("  W        ", kh), Span::styled("  go to workspace picker", theme.muted())]),
        Line::from(""),
        Line::from(Span::styled("  Response", theme.title())),
        Line::from(vec![Span::styled("  1 2 3 4  ", kh), Span::styled("  Body / Headers / Logs / History", theme.muted())]),
        Line::from(vec![Span::styled("  Tab      ", kh), Span::styled("  cycle response tabs", theme.muted())]),
        Line::from(vec![Span::styled("  PgUp/Dn  ", kh), Span::styled("  scroll response body", theme.muted())]),
        Line::from(""),
        Line::from(Span::styled("  Other", theme.title())),
        Line::from(vec![Span::styled("  e        ", kh), Span::styled("  cycle environment", theme.muted())]),
        Line::from(vec![Span::styled("  c / Esc  ", kh), Span::styled("  clear error / dismiss overlays", theme.muted())]),
        Line::from(vec![Span::styled("  ?        ", kh), Span::styled("  toggle help", theme.muted())]),
        Line::from(vec![Span::styled("  q        ", kh), Span::styled("  quit", theme.muted())]),
    ];

    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.primary))
                    .title(" Help ")
                    .title_style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
                    .style(Style::default().bg(theme.panel_bg)),
            )
            .wrap(Wrap { trim: true }),
        popup,
    );
}

fn render_error_overlay(frame: &mut Frame<'_>, area: Rect, error: &str, theme: &Theme) {
    let popup = centered_rect(72, 30, area);

    let content = vec![
        Line::from(Span::styled(
            "  Request Error",
            Style::default().fg(theme.error_color).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(error.to_string(), theme.body())),
        Line::from(""),
        Line::from(Span::styled("  Press  c  or  Esc  to dismiss", theme.muted())),
    ];

    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.error_color))
                    .title(" Error ")
                    .title_style(Style::default().fg(theme.error_color).add_modifier(Modifier::BOLD))
                    .style(Style::default().bg(theme.panel_bg)),
            )
            .wrap(Wrap { trim: true }),
        popup,
    );
}

// ─── Utilities ───────────────────────────────────────────────────────────────

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vert = Layout::default()
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
        .split(vert[1])[1]
}

fn auth_label(req: &crate::core::models::RequestTemplate) -> String {
    match &req.auth {
        crate::core::models::AuthStrategy::None                     => "none".to_string(),
        crate::core::models::AuthStrategy::Basic { username, .. }  => format!("basic ({})", username),
        crate::core::models::AuthStrategy::Bearer { .. }           => "bearer".to_string(),
        crate::core::models::AuthStrategy::ApiKey { key, in_header, .. } => {
            if *in_header { format!("api-key header ({})", key) }
            else          { format!("api-key query ({})", key)  }
        }
    }
}

fn format_size(bytes: usize) -> String {
    if bytes < 1024            { format!("{} B",    bytes) }
    else if bytes < 1024*1024  { format!("{:.1} KB", bytes as f64 / 1024.0) }
    else                        { format!("{:.2} MB", bytes as f64 / (1024.0*1024.0)) }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..s.char_indices().nth(max-1).map(|(i,_)|i).unwrap_or(s.len())])
    }
}

