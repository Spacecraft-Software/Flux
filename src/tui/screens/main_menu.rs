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

    // Header
    let header = Paragraph::new("Flux — DNS Selector & Network Configurator")
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

    // Menu
    let items: Vec<ListItem> = App::menu_items()
        .iter()
        .enumerate()
        .map(|(i, &item)| {
            let style = if i == app.menu_idx {
                Style::default()
                    .fg(Palette::VOID_NAVY)
                    .bg(Palette::STEEL_BLUE)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Palette::MOLTEN_AMBER)
            };
            ListItem::new(Line::from(Span::styled(format!("  {item}"), style)))
        })
        .collect();

    let menu = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Palette::STEEL_BLUE))
            .title("Main Menu"),
    );
    f.render_widget(menu, chunks[1]);

    // Footer
    let footer = Paragraph::new("j/k ↑/↓  Enter select  q quit")
        .style(Style::default().fg(Palette::LIQUID_COOLANT))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}

pub fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) -> Option<Screen> {
    use crossterm::event::KeyCode;
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => return Some(Screen::About),
        KeyCode::Char('j') | KeyCode::Down => {
            app.menu_idx = (app.menu_idx + 1) % App::menu_items().len();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.menu_idx == 0 {
                app.menu_idx = App::menu_items().len() - 1;
            } else {
                app.menu_idx -= 1;
            }
        }
        KeyCode::Enter => {
            return match app.menu_idx {
                0 => Some(Screen::ProviderSelect),
                3 => Some(Screen::Status),
                5 => Some(Screen::About),
                7 => return None, // Quit
                _ => None,
            };
        }
        _ => {}
    }
    None
}
