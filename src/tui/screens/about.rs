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

pub fn draw(f: &mut Frame, _app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.area());

    let header = Paragraph::new("About Flux")
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

    let text = vec![
        Line::from(Span::styled(
            "Flux",
            Style::default()
                .fg(Palette::MOLTEN_AMBER)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "DNS Selector & Network Configurator",
            Style::default().fg(Palette::MOLTEN_AMBER),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Version: 0.1.0",
            Style::default().fg(Palette::LIQUID_COOLANT),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Maintainer:",
            Style::default().fg(Palette::STEEL_BLUE),
        )),
        Line::from(Span::styled(
            "Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>",
            Style::default().fg(Palette::MOLTEN_AMBER),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Project URL:",
            Style::default().fg(Palette::STEEL_BLUE),
        )),
        Line::from(Span::styled(
            "https://Flux.SpacecraftSoftware.org/",
            Style::default().fg(Palette::LIQUID_COOLANT),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "License: GPL-3.0-or-later",
            Style::default().fg(Palette::MOLTEN_AMBER),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "© 2026 Mohamed Hammad",
            Style::default().fg(Palette::MOLTEN_AMBER),
        )),
    ];

    let about = Paragraph::new(text).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Palette::STEEL_BLUE)),
    );
    f.render_widget(about, chunks[1]);

    let footer = Paragraph::new("Esc or q to return")
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
