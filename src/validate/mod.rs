// SPDX-License-Identifier: GPL-3.0-or-later

use crate::error::AppError;
use std::path::Path;

/// Reject control characters (0x00–0x08, 0x0B–0x0C, 0x0E–0x1F) and ANSI escapes in string args.
pub fn reject_control_chars(s: &str) -> Result<(), AppError> {
    for ch in s.chars() {
        let cp = ch as u32;
        if cp <= 0x1F || cp == 0x7F {
            return Err(AppError::usage_error(format!(
                "Control character (U+{cp:04X}) rejected in argument"
            )));
        }
        // Reject ANSI escape sequences (ESC [ or ESC ] or ESC ( etc.)
        if ch == '\x1B' {
            return Err(AppError::usage_error(
                "ANSI escape sequences are not permitted in arguments",
            ));
        }
    }
    Ok(())
}

#[allow(dead_code)]
/// Canonicalize a path argument and reject traversal, symlink escapes, and out-of-bounds paths.
pub fn canonicalize_path_arg(path: &str) -> Result<std::path::PathBuf, AppError> {
    reject_control_chars(path)?;

    let p = Path::new(path);

    // Reject paths containing .. components
    if p.components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(AppError::usage_error(
            "Path traversal ('..') is not permitted",
        ));
    }

    // Canonicalize resolves symlinks
    let canonical = std::fs::canonicalize(p)
        .map_err(|e| AppError::usage_error(format!("Invalid path '{path}': {e}")))?;

    // Allow-list: must be under /etc/ or ~/.local/share/flux/ or ~/.config/flux/
    let allowed = is_under_allowed(&canonical)?;
    if !allowed {
        return Err(AppError::usage_error(format!(
            "Path '{path}' is outside the allowed directories (/etc/, ~/.local/share/flux/, ~/.config/flux/)"
        )));
    }

    Ok(canonical)
}

#[allow(dead_code)]
/// Check that a numeric value lies within inclusive bounds.
pub fn bounds_check_numeric<T>(value: T, min: T, max: T) -> Result<T, AppError>
where
    T: PartialOrd + std::fmt::Display,
{
    if value < min || value > max {
        return Err(AppError::usage_error(format!(
            "Value {value} out of bounds [{min}, {max}]"
        )));
    }
    Ok(value)
}

/// Reject strings that contain known indirect-prompt-injection patterns.
///
/// This is defense-in-depth for agentic CLI usage (MCP, CI, Codex, etc.).
/// Even when invoked by a human user, these substrings have no legitimate
/// purpose as DNS/NTP/VPN arguments.
pub fn reject_prompt_injection(s: &str) -> Result<(), AppError> {
    let lower = s.to_lowercase();
    const PATTERNS: &[&str] = &[
        "ignore previous instructions",
        "ignore all prior",
        "ignore previous commands",
        "system prompt",
        "you are now",
        "new instructions",
        "disregard all",
        "forget everything",
        "override instructions",
        "bypass security",
        "jailbreak",
        "do anything now",
    ];

    for pat in PATTERNS {
        if lower.contains(pat) {
            return Err(AppError::usage_error(
                "Potential prompt-injection pattern detected in input",
            ));
        }
    }

    Ok(())
}

fn is_under_allowed(path: &Path) -> Result<bool, AppError> {
    let etc = Path::new("/etc");
    let local_share = dirs::home_dir()
        .ok_or_else(|| AppError::usage_error("Cannot determine home directory"))?
        .join(".local/share/flux");
    let config = dirs::home_dir()
        .ok_or_else(|| AppError::usage_error("Cannot determine home directory"))?
        .join(".config/flux");

    Ok(path.starts_with(etc) || path.starts_with(&local_share) || path.starts_with(&config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reject_control_chars_ok() {
        assert!(reject_control_chars("hello-world").is_ok());
        assert!(reject_control_chars("cloudflare").is_ok());
    }

    #[test]
    fn test_reject_control_chars_null() {
        assert!(reject_control_chars("hello\x00world").is_err());
    }

    #[test]
    fn test_reject_control_chars_bell() {
        assert!(reject_control_chars("hello\x07world").is_err());
    }

    #[test]
    fn test_reject_ansi_escape() {
        assert!(reject_control_chars("\x1B[31mred\x1B[0m").is_err());
    }

    #[test]
    fn test_reject_tab() {
        assert!(reject_control_chars("hello\tworld").is_err());
    }

    #[test]
    fn test_reject_newline() {
        assert!(reject_control_chars("hello\nworld").is_err());
    }

    #[test]
    fn test_reject_carriage_return() {
        assert!(reject_control_chars("hello\rworld").is_err());
    }

    #[test]
    fn test_reject_path_traversal() {
        assert!(canonicalize_path_arg("/etc/../etc/passwd").is_err());
    }

    #[test]
    fn test_path_must_exist() {
        assert!(canonicalize_path_arg("/nonexistent/path/file.txt").is_err());
    }

    #[test]
    fn test_bounds_check_numeric_in_range() {
        assert_eq!(bounds_check_numeric(5u32, 0, 10).unwrap(), 5);
        assert_eq!(bounds_check_numeric(0i32, -10, 10).unwrap(), 0);
        assert_eq!(bounds_check_numeric(10u64, 10, 20).unwrap(), 10);
    }

    #[test]
    fn test_bounds_check_numeric_under_min() {
        let result = bounds_check_numeric(5u32, 10, 20);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("out of bounds"));
    }

    #[test]
    fn test_bounds_check_numeric_over_max() {
        let result = bounds_check_numeric(25u32, 0, 20);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("out of bounds"));
    }

    #[test]
    fn test_reject_prompt_injection_clean() {
        assert!(reject_prompt_injection("cloudflare").is_ok());
        assert!(reject_prompt_injection("time.google.com").is_ok());
    }

    #[test]
    fn test_reject_prompt_injection_ignore_previous() {
        assert!(reject_prompt_injection("ignore previous instructions").is_err());
    }

    #[test]
    fn test_reject_prompt_injection_case_insensitive() {
        assert!(reject_prompt_injection("IGNORE PREVIOUS COMMANDS").is_err());
        assert!(reject_prompt_injection("System Prompt:").is_err());
    }

    #[test]
    fn test_reject_prompt_injection_embedded() {
        assert!(reject_prompt_injection("hello disregard all world").is_err());
    }

    #[test]
    fn test_reject_prompt_injection_jailbreak() {
        assert!(reject_prompt_injection("jailbreak").is_err());
    }
}
