// SPDX-License-Identifier: GPL-3.0-or-later

pub mod csv;
pub mod envelope;
pub mod human;
pub mod mode;
pub mod trim;
pub mod yaml;

use std::io::{self, Write};

/// Returns current UTC timestamp in ISO 8601 with mandatory `Z` suffix.
/// Format: YYYY-MM-DDTHH:MM:SSZ
pub fn now_utc() -> String {
    jiff::Timestamp::now()
        .strftime("%Y-%m-%dT%H:%M:%SZ")
        .to_string()
}

/// Emit structured data to stdout in the selected machine format.
pub fn emit_data<T: serde::Serialize>(data: &T, mode: &mode::OutputMode) {
    match mode.format {
        mode::Format::Json => {
            let envelope = envelope::Response::new(payload_value(data, mode));
            let json = serde_json::to_string(&envelope).unwrap_or_else(|_| "{}".to_string());
            let _ = writeln!(io::stdout(), "{json}");
        }
        mode::Format::Jsonl => {
            // Newline-delimited JSON: one bare record per line (no repeated
            // envelope) so list output streams and stays cheap for agents
            // (ACS §8). A non-array payload is emitted as a single line.
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            match payload_value(data, mode) {
                serde_json::Value::Array(items) => {
                    for item in &items {
                        let line = serde_json::to_string(item).unwrap_or_else(|_| "{}".to_string());
                        let _ = writeln!(handle, "{line}");
                    }
                }
                other => {
                    let line = serde_json::to_string(&other).unwrap_or_else(|_| "{}".to_string());
                    let _ = writeln!(handle, "{line}");
                }
            }
        }
        mode::Format::Yaml => {
            // Mirror the JSON `{ metadata, data }` envelope, then render as YAML.
            let envelope = envelope::Response::new(payload_value(data, mode));
            let value = serde_json::to_value(&envelope).unwrap_or(serde_json::Value::Null);
            // `to_yaml` already terminates with a newline.
            let _ = write!(io::stdout(), "{}", yaml::to_yaml(&value));
        }
        mode::Format::Csv => {
            // CSV is tabular: emit the data payload only (the envelope has no row form).
            let _ = write!(io::stdout(), "{}", csv::to_csv(&payload_value(data, mode)));
        }
        mode::Format::Human | mode::Format::Explore => {
            // Human output handled per-command.
        }
    }
}

/// Serialize `data` to a JSON value, applying `--fields` selection when present.
fn payload_value<T: serde::Serialize>(data: &T, mode: &mode::OutputMode) -> serde_json::Value {
    let value = serde_json::to_value(data).unwrap_or(serde_json::Value::Null);
    match mode.fields {
        Some(ref fields) => trim::trim_payload(value, fields),
        None => value,
    }
}

/// Emit structured error to stderr.
pub fn emit_error(err: &crate::error::AppError, mode: &mode::OutputMode) {
    match mode.format {
        // All machine formats emit the structured error as JSON on stderr;
        // stdout stays reserved for data (PRD §9.7).
        mode::Format::Json | mode::Format::Jsonl | mode::Format::Yaml | mode::Format::Csv => {
            let json = serde_json::to_string(err).unwrap_or_else(|_| "{}".to_string());
            let _ = writeln!(io::stderr(), "{json}");
        }
        mode::Format::Human | mode::Format::Explore => {
            let rendered = human::render_error(err, mode.color);
            let _ = writeln!(io::stderr(), "{rendered}");
        }
    }
}
