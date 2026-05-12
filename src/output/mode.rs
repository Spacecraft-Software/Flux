// SPDX-License-Identifier: GPL-3.0-or-later

use std::io;

/// Output format variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Json,
    Jsonl,
    Yaml,
    Csv,
    Human,
    Explore,
}

/// Computed output mode shared by every subcommand.
#[derive(Debug, Clone)]
pub struct OutputMode {
    pub format: Format,
    pub color: ColorMode,
    pub interactive: bool,
    #[allow(dead_code)]
    pub verbose: bool,
    #[allow(dead_code)]
    pub quiet: bool,
    pub dry_run: bool,
    #[allow(dead_code)]
    pub yes: bool,
    pub fields: Option<Vec<String>>,
    #[allow(dead_code)]
    pub absolute_time: bool,
    #[allow(dead_code)]
    pub print0: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Never,
    Always,
    Auto,
}

impl OutputMode {
    /// Compute output mode from CLI flags and environment.
    ///
    /// Cascade order (SFRS §5):
    /// 1. Explicit --format / --json flag
    /// 2. Agent env: AI_AGENT=1, AGENT=1, CI=true → JSON, no-color, no-TUI, non-interactive
    /// 3. TTY → human mode + color
    /// 4. Non-TTY → JSON mode
    /// 5. Fallback → human mode
    #[allow(clippy::too_many_arguments)]
    pub fn from_args(
        format_flag: Option<FormatArg>,
        json_flag: bool,
        no_color_flag: bool,
        color_flag: Option<ColorArg>,
        dry_run: bool,
        verbose: bool,
        quiet: bool,
        yes: bool,
        fields: Option<String>,
        absolute_time: bool,
        print0: bool,
    ) -> Self {
        let is_tty = std::io::IsTerminal::is_terminal(&io::stdout());

        // Color precedence: NO_COLOR > FORCE_COLOR > CLICOLOR > --color > --no-color > TTY detection
        let color = if std::env::var("NO_COLOR").is_ok_and(|v| !v.is_empty()) {
            ColorMode::Never
        } else if std::env::var("FORCE_COLOR").is_ok_and(|v| !v.is_empty()) {
            ColorMode::Always
        } else if let Ok(v) = std::env::var("CLICOLOR") {
            if v == "0" {
                ColorMode::Never
            } else {
                ColorMode::Auto
            }
        } else if no_color_flag {
            ColorMode::Never
        } else if let Some(c) = color_flag {
            c.into()
        } else if std::env::var("TERM").is_ok_and(|v| v.eq_ignore_ascii_case("dumb")) {
            ColorMode::Never
        } else if is_tty {
            ColorMode::Auto
        } else {
            ColorMode::Never
        };

        let agent_env = std::env::var("AI_AGENT").is_ok_and(|v| v == "1")
            || std::env::var("AGENT").is_ok_and(|v| v == "1")
            || std::env::var("CI").is_ok_and(|v| v.eq_ignore_ascii_case("true"));

        let term_dumb = std::env::var("TERM").is_ok_and(|v| v.eq_ignore_ascii_case("dumb"));

        let (format, interactive) = if json_flag {
            (Format::Json, false)
        } else if let Some(f) = format_flag {
            let fmt = f.into();
            // Explore guard: if AI_AGENT=1 and --format explore requested, fall back to JSON
            if agent_env && fmt == Format::Explore {
                eprintln!("Warning: TUI suppressed in agent mode; falling back to JSON");
                (Format::Json, false)
            } else {
                let interactive = fmt == Format::Explore || fmt == Format::Human;
                (fmt, interactive && is_tty && !agent_env && !term_dumb)
            }
        } else if agent_env {
            (Format::Json, false)
        } else if is_tty {
            (Format::Human, true)
        } else {
            (Format::Json, false)
        };

        let fields = fields.map(|s| s.split(',').map(|s| s.trim().to_string()).collect());

        Self {
            format,
            color,
            interactive,
            verbose,
            quiet,
            dry_run,
            yes: yes || agent_env,
            fields,
            absolute_time,
            print0,
        }
    }

    #[allow(dead_code)]
    pub fn is_machine(&self) -> bool {
        matches!(
            self.format,
            Format::Json | Format::Jsonl | Format::Yaml | Format::Csv
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum FormatArg {
    Json,
    Jsonl,
    Yaml,
    Csv,
    Human,
    Explore,
}

impl From<FormatArg> for Format {
    fn from(arg: FormatArg) -> Self {
        match arg {
            FormatArg::Json => Format::Json,
            FormatArg::Jsonl => Format::Jsonl,
            FormatArg::Yaml => Format::Yaml,
            FormatArg::Csv => Format::Csv,
            FormatArg::Human => Format::Human,
            FormatArg::Explore => Format::Explore,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ColorArg {
    Never,
    Always,
    Auto,
}

impl From<ColorArg> for ColorMode {
    fn from(arg: ColorArg) -> Self {
        match arg {
            ColorArg::Never => ColorMode::Never,
            ColorArg::Always => ColorMode::Always,
            ColorArg::Auto => ColorMode::Auto,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_env<F>(vars: &[(&str, &str)], f: F) -> OutputMode
    where
        F: FnOnce() -> OutputMode,
    {
        // Set env vars
        for (k, v) in vars {
            // SAFETY: single-threaded test context
            unsafe {
                std::env::set_var(k, v);
            }
        }
        let result = f();
        // Clean up
        for (k, _v) in vars {
            // SAFETY: single-threaded test context
            unsafe {
                std::env::remove_var(k);
            }
        }
        result
    }

    #[test]
    fn test_agent_env_forces_json() {
        let mode = with_env(&[("AI_AGENT", "1")], || {
            OutputMode::from_args(
                None, false, false, None, false, false, false, false, None, false, false,
            )
        });
        assert_eq!(mode.format, Format::Json);
        assert!(!mode.interactive);
        assert_eq!(mode.color, ColorMode::Never);
    }

    #[test]
    fn test_explicit_format_overrides() {
        let mode = OutputMode::from_args(
            Some(FormatArg::Yaml),
            false,
            false,
            None,
            false,
            false,
            false,
            false,
            None,
            false,
            false,
        );
        assert_eq!(mode.format, Format::Yaml);
    }

    #[test]
    fn test_json_flag() {
        let mode = OutputMode::from_args(
            None, true, false, None, false, false, false, false, None, false, false,
        );
        assert_eq!(mode.format, Format::Json);
    }

    #[test]
    fn test_no_color_env() {
        let mode = with_env(&[("NO_COLOR", "1")], || {
            OutputMode::from_args(
                None, false, false, None, false, false, false, false, None, false, false,
            )
        });
        assert_eq!(mode.color, ColorMode::Never);
    }

    #[test]
    fn test_force_color_env() {
        let mode = with_env(&[("FORCE_COLOR", "1")], || {
            OutputMode::from_args(
                None, false, false, None, false, false, false, false, None, false, false,
            )
        });
        assert_eq!(mode.color, ColorMode::Always);
    }

    #[test]
    fn test_explore_guard_with_agent() {
        let mode = with_env(&[("AI_AGENT", "1")], || {
            OutputMode::from_args(
                Some(FormatArg::Explore),
                false,
                false,
                None,
                false,
                false,
                false,
                false,
                None,
                false,
                false,
            )
        });
        assert_eq!(mode.format, Format::Json);
    }

    #[test]
    fn test_fields_parsing() {
        let mode = OutputMode::from_args(
            None,
            false,
            false,
            None,
            false,
            false,
            false,
            false,
            Some("slug,protocols".to_string()),
            false,
            false,
        );
        assert_eq!(
            mode.fields,
            Some(vec!["slug".to_string(), "protocols".to_string()])
        );
    }
}
