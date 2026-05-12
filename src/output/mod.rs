// SPDX-License-Identifier: GPL-3.0-or-later

pub mod envelope;
pub mod human;
pub mod mode;
pub mod trim;

use std::io::{self, Write};

/// Returns current UTC timestamp in ISO 8601 with mandatory `Z` suffix.
/// Format: YYYY-MM-DDTHH:MM:SSZ
pub fn now_utc() -> String {
    jiff::Timestamp::now()
        .strftime("%Y-%m-%dT%H:%M:%SZ")
        .to_string()
}

/// Emit structured data to stdout as JSON envelope.
pub fn emit_data<T: serde::Serialize>(data: &T, mode: &mode::OutputMode) {
    match mode.format {
        mode::Format::Json | mode::Format::Jsonl => {
            let mut value = serde_json::to_value(data).unwrap_or(serde_json::Value::Null);
            if let Some(ref fields) = mode.fields {
                value = trim::trim_value(value, fields);
            }
            let envelope = envelope::Response::new(value);
            let json = serde_json::to_string(&envelope).unwrap_or_else(|_| "{}".to_string());
            let _ = writeln!(io::stdout(), "{json}");
        }
        mode::Format::Yaml => {
            // TODO: implement YAML output
            let _ = writeln!(io::stdout(), "# YAML output not yet implemented");
        }
        mode::Format::Csv => {
            // TODO: implement CSV output
            let _ = writeln!(io::stdout(), "# CSV output not yet implemented");
        }
        mode::Format::Human | mode::Format::Explore => {
            // Human output handled per-command
        }
    }
}

/// Emit structured error to stderr.
pub fn emit_error(err: &crate::error::AppError, mode: &mode::OutputMode) {
    match mode.format {
        mode::Format::Json | mode::Format::Jsonl => {
            let json = serde_json::to_string(err).unwrap_or_else(|_| "{}".to_string());
            let _ = writeln!(io::stderr(), "{json}");
        }
        _ => {
            let rendered = human::render_error(err, mode.color);
            let _ = writeln!(io::stderr(), "{rendered}");
        }
    }
}
