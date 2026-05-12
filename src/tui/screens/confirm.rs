// SPDX-License-Identifier: GPL-3.0-or-later

use crate::tui::app::{App, Screen};
use crate::tui::theme::Palette;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(f.area());

    let header = Paragraph::new("Confirm Application")
        .style(
            Style::default()
                .fg(Palette::STEEL_BLUE)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Palette::STEEL_BLUE)),
        );
    f.render_widget(header, chunks[0]);

    let provider = app.selected_provider.as_deref().unwrap_or("?");
    let tier = app.selected_tier.map(|t| t.to_string()).unwrap_or_default();
    let protocol = app
        .selected_protocol
        .map(|p| p.to_string())
        .unwrap_or_default();
    let backend = app
        .detected_backend
        .map(|b| b.to_string())
        .unwrap_or_default();

    let text = vec![
        Line::from(Span::styled(
            "Summary:",
            Style::default()
                .fg(Palette::STEEL_BLUE)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  Provider:  {provider}"),
            Style::default().fg(Palette::MOLTEN_AMBER),
        )),
        Line::from(Span::styled(
            format!("  Tier:      {tier}"),
            Style::default().fg(Palette::MOLTEN_AMBER),
        )),
        Line::from(Span::styled(
            format!("  Protocol:  {protocol}"),
            Style::default().fg(Palette::MOLTEN_AMBER),
        )),
        Line::from(Span::styled(
            format!("  Backend:   {backend}"),
            Style::default().fg(Palette::MOLTEN_AMBER),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Apply this configuration?",
            Style::default().fg(Palette::RADIUM_GREEN),
        )),
        Line::from(Span::styled(
            "y = yes, n = no",
            Style::default().fg(Palette::LIQUID_COOLANT),
        )),
    ];

    let summary = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Palette::STEEL_BLUE)),
    );
    f.render_widget(summary, chunks[1]);

    let footer = Paragraph::new("y confirm  n cancel  Esc back")
        .style(Style::default().fg(Palette::LIQUID_COOLANT))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}

pub fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) -> Option<Screen> {
    use crossterm::event::KeyCode;
    match key.code {
        KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
            return Some(Screen::ProtocolSelect);
        }
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.confirm = true;
            match app.apply_selection() {
                Ok(msg) => {
                    app.status_message = Some(msg);
                }
                Err(e) => {
                    app.status_message = Some(format!("Error: {}", e.message));
                }
            }
            return Some(Screen::Status);
        }
        _ => {}
    }
    None
}
