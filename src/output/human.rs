// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{self, IsTerminal};

use crate::error::AppError;
use crate::output::mode::ColorMode;

/// Steelbore v1.2 six-token palette — ANSI 24-bit foreground sequences.
const RED_OXIDE: &str = "\x1b[38;2;255;92;92m";
const MOLTEN_AMBER: &str = "\x1b[38;2;217;142;50m";
const LIQUID_COOLANT: &str = "\x1b[38;2;139;233;253m";
const RESET: &str = "\x1b[0m";

/// Determine whether ANSI color codes should be emitted.
fn color_enabled(mode: ColorMode) -> bool {
    match mode {
        ColorMode::Never => false,
        ColorMode::Always => true,
        ColorMode::Auto => io::stderr().is_terminal(),
    }
}

/// Wrap a string in an ANSI 24-bit color sequence when `enabled`.
fn paint(text: &str, color: &str, enabled: bool) -> String {
    if enabled {
        format!("{color}{text}{RESET}")
    } else {
        text.to_string()
    }
}

/// Render an `AppError` for human consumption.
///
/// - Error code in **Red Oxide** `#FF5C5C`
/// - Message in **Molten Amber** `#D98E32`
/// - Hint command in **Liquid Coolant** `#8BE9FD`
pub fn render_error(err: &AppError, color: ColorMode) -> String {
    let enabled = color_enabled(color);
    let code = paint(&err.code, RED_OXIDE, enabled);
    let message = paint(&err.message, MOLTEN_AMBER, enabled);
    let hint = paint(&err.hint, LIQUID_COOLANT, enabled);
    format!("Error [{code}]: {message}\nHint: {hint}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{AppError, ExitCode};

    #[test]
    fn test_colored_output_contains_ansi() {
        let err = AppError::new(
            ExitCode::ElevationFailed,
            "No elevation tool found",
            "sudo dns apply ...",
        );
        let rendered = render_error(&err, ColorMode::Always);
        assert!(rendered.contains(RED_OXIDE), "expected Red Oxide ANSI code");
        assert!(
            rendered.contains(MOLTEN_AMBER),
            "expected Molten Amber ANSI code"
        );
        assert!(
            rendered.contains(LIQUID_COOLANT),
            "expected Liquid Coolant ANSI code"
        );
        assert!(rendered.contains(RESET), "expected RESET ANSI code");
    }

    #[test]
    fn test_plain_output_no_ansi() {
        let err = AppError::new(
            ExitCode::ElevationFailed,
            "No elevation tool found",
            "sudo dns apply ...",
        );
        let rendered = render_error(&err, ColorMode::Never);
        assert!(
            !rendered.contains('\x1b'),
            "expected no ANSI escape sequences"
        );
    }

    #[test]
    fn test_hint_is_runnable_command() {
        let err = AppError::new(
            ExitCode::DetectionFailed,
            "Backend detection failed",
            "dns detect --json",
        );
        let rendered = render_error(&err, ColorMode::Never);
        assert!(rendered.contains("dns detect --json"));
        assert!(rendered.starts_with("Error [DetectionFailed]: Backend detection failed"));
    }
}
