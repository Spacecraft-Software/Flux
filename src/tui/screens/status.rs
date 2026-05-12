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
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.area());

    let header = Paragraph::new("Status")
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

    let mut lines = vec![
        Line::from(Span::styled(
            "Result:",
            Style::default()
                .fg(Palette::STEEL_BLUE)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    if let Some(msg) = &app.status_message {
        lines.push(Line::from(Span::styled(
            msg.clone(),
            Style::default().fg(Palette::RADIUM_GREEN),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "No status available.",
            Style::default().fg(Palette::MOLTEN_AMBER),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(
            "Detected backend: {}",
            app.detected_backend
                .map(|b| b.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        ),
        Style::default().fg(Palette::LIQUID_COOLANT),
    )));

    let status = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Palette::STEEL_BLUE)),
    );
    f.render_widget(status, chunks[1]);

    let footer = Paragraph::new("Esc or q to return to main menu")
        .style(Style::default().fg(Palette::LIQUID_COOLANT))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}

pub fn handle_key(_app: &mut App, key: crossterm::event::KeyEvent) -> Option<Screen> {
    use crossterm::event::KeyCode;
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => Some(Screen::MainMenu),
        _ => None,
    }
}
