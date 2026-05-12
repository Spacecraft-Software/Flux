// SPDX-License-Identifier: GPL-3.0-or-later

use crate::tui::app::{App, Screen};
use crate::tui::theme::Palette;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
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

    let header = Paragraph::new("Select Protocol")
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

    let protocols = app.protocol_items();
    let items: Vec<ListItem> = protocols
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let style = if i == app.protocol_idx {
                Style::default()
                    .fg(Palette::VOID_NAVY)
                    .bg(Palette::STEEL_BLUE)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Palette::MOLTEN_AMBER)
            };
            ListItem::new(Line::from(Span::styled(format!("  {name}"), style)))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Palette::STEEL_BLUE)),
    );
    f.render_widget(list, chunks[1]);

    let footer = Paragraph::new("j/k ↑/↓  Enter select  Esc back")
        .style(Style::default().fg(Palette::LIQUID_COOLANT))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}

pub fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) -> Option<Screen> {
    use crossterm::event::KeyCode;
    let protocols = app.protocol_items();
    match key.code {
        KeyCode::Esc => return Some(Screen::TierSelect),
        KeyCode::Char('j') | KeyCode::Down => {
            app.protocol_idx =
                std::cmp::min(app.protocol_idx + 1, protocols.len().saturating_sub(1));
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.protocol_idx = app.protocol_idx.saturating_sub(1);
        }
        KeyCode::Enter => {
            if let Some(name) = protocols.get(app.protocol_idx) {
                app.selected_protocol = name.to_lowercase().parse().ok();
                return Some(Screen::Confirmation);
            }
        }
        _ => {}
    }
    None
}
