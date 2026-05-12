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

    let header = Paragraph::new("Select DNS Provider")
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

    let providers = App::provider_items();
    let items: Vec<ListItem> = providers
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let style = if i == app.provider_idx {
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
    let providers = App::provider_items();
    match key.code {
        KeyCode::Esc => return Some(Screen::MainMenu),
        KeyCode::Char('j') | KeyCode::Down => {
            app.provider_idx =
                std::cmp::min(app.provider_idx + 1, providers.len().saturating_sub(1));
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.provider_idx = app.provider_idx.saturating_sub(1);
        }
        KeyCode::Enter => {
            if let Some(name) = providers.get(app.provider_idx) {
                // Map display name back to slug
                let slug = crate::registry::list_providers()
                    .into_iter()
                    .find(|p| p.name == *name)
                    .map(|p| p.slug.clone());
                app.selected_provider = slug;
                return Some(Screen::TierSelect);
            }
        }
        _ => {}
    }
    None
}
