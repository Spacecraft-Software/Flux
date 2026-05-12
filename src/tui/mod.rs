// SPDX-License-Identifier: GPL-3.0-or-later

pub mod app;
pub mod screens;
pub mod theme;

use app::{App, Screen};
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend, style::Style};
use std::io::stdout;

/// Run the TUI interface.
pub fn run_tui() -> Result<(), crate::error::AppError> {
    enable_raw_mode()
        .map_err(|e| crate::error::AppError::general(format!("Terminal error: {e}")))?;
    stdout()
        .execute(EnterAlternateScreen)
        .map_err(|e| crate::error::AppError::general(format!("Terminal error: {e}")))?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))
        .map_err(|e| crate::error::AppError::general(format!("Terminal error: {e}")))?;

    let mut app = App::new();
    let mut should_quit = false;

    let result = loop {
        if should_quit {
            break Ok(());
        }

        if let Err(e) = terminal.draw(|f| {
            f.render_widget(
                ratatui::widgets::Block::default()
                    .style(Style::default().bg(theme::Palette::VOID_NAVY)),
                f.area(),
            );
            match app.screen {
                Screen::MainMenu => screens::main_menu::draw(f, &app),
                Screen::ProviderSelect => screens::provider::draw(f, &app),
                Screen::TierSelect => screens::tier::draw(f, &app),
                Screen::ProtocolSelect => screens::protocol::draw(f, &app),
                Screen::Confirmation => screens::confirm::draw(f, &app),
                Screen::Status => screens::status::draw(f, &app),
                Screen::About => screens::about::draw(f, &app),
                Screen::Progress => {}
            }
        }) {
            break Err(crate::error::AppError::general(format!("Draw error: {e}")));
        }

        if let Ok(true) = event::poll(std::time::Duration::from_millis(100)) {
            if let Ok(Event::Key(key)) = event::read() {
                // Global quit
                if key.code == KeyCode::Char('c')
                    && key.modifiers.contains(event::KeyModifiers::CONTROL)
                {
                    should_quit = true;
                    continue;
                }

                let next = match app.screen {
                    Screen::MainMenu => screens::main_menu::handle_key(&mut app, key),
                    Screen::ProviderSelect => screens::provider::handle_key(&mut app, key),
                    Screen::TierSelect => screens::tier::handle_key(&mut app, key),
                    Screen::ProtocolSelect => screens::protocol::handle_key(&mut app, key),
                    Screen::Confirmation => screens::confirm::handle_key(&mut app, key),
                    Screen::Status => screens::status::handle_key(&mut app, key),
                    Screen::About => screens::about::handle_key(&mut app, key),
                    Screen::Progress => None,
                };

                if let Some(screen) = next {
                    app.screen = screen;
                } else if app.screen == Screen::MainMenu && key.code == KeyCode::Char('q') {
                    should_quit = true;
                }
            }
        }
    };

    let _ = disable_raw_mode();
    let _ = stdout().execute(LeaveAlternateScreen);
    result
}
