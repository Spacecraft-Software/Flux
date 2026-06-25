// SPDX-License-Identifier: GPL-3.0-or-later
// Rust guideline compliant 2026-05-18

//! Minimal YAML 1.2 block-style serializer for [`serde_json::Value`] trees.
//!
//! Flux emits its `{ metadata, data }` envelope in several machine formats.
//! Rather than depend on an unmaintained YAML crate — which would trip the
//! mandatory `cargo audit` CI gate (Steelbore Standard §3.3) — we render the
//! already-built JSON value tree directly. The value model is closed (only the
//! six `serde_json::Value` variants), so a small, fully-tested emitter is
//! sufficient and adds zero dependency-audit surface.

use serde_json::{Map, Value};

/// Width, in spaces, of one block-indent level.
const INDENT_STEP: usize = 2;

/// Render a JSON value as a YAML 1.2 document in block style.
///
/// The output is terminated by a single trailing newline. Strings are quoted
/// only when a plain scalar would be ambiguous (numbers, booleans, nulls,
/// reserved words, or values containing YAML-significant characters).
pub fn to_yaml(value: &Value) -> String {
    let mut out = String::new();
    match value {
        Value::Object(map) if !map.is_empty() => write_mapping(&mut out, map, 0, false),
        Value::Array(items) if !items.is_empty() => write_sequence(&mut out, items, 0),
        // Top-level scalar or empty container renders on a single line.
        other => {
            out.push_str(&scalar(other));
            out.push('\n');
        }
    }
    out
}

/// Write a block mapping at `indent`.
///
/// When `inline_first` is set the first key is emitted without leading
/// indentation, because the caller has already written a `- ` sequence marker
/// on the current line.
fn write_mapping(out: &mut String, map: &Map<String, Value>, indent: usize, inline_first: bool) {
    let mut inline = inline_first;
    for (key, val) in map {
        if inline {
            inline = false;
        } else {
            pad(out, indent);
        }
        out.push_str(&scalar_string(key));
        out.push(':');
        write_after_marker(out, val, indent);
    }
}

/// Write a block sequence at `indent`.
fn write_sequence(out: &mut String, items: &[Value], indent: usize) {
    for item in items {
        match item {
            Value::Object(map) if !map.is_empty() => {
                pad(out, indent);
                out.push_str("- ");
                // The first key shares the dash line; the rest align under it.
                write_mapping(out, map, indent + INDENT_STEP, true);
            }
            Value::Array(inner) if !inner.is_empty() => {
                pad(out, indent);
                out.push_str("-\n");
                write_sequence(out, inner, indent + INDENT_STEP);
            }
            other => {
                pad(out, indent);
                out.push_str("- ");
                out.push_str(&scalar(other));
                out.push('\n');
            }
        }
    }
}

/// Write the value that follows a `key:` or `- ` marker, choosing inline scalar
/// form for leaves and block form for non-empty containers.
fn write_after_marker(out: &mut String, val: &Value, indent: usize) {
    match val {
        Value::Object(map) if !map.is_empty() => {
            out.push('\n');
            write_mapping(out, map, indent + INDENT_STEP, false);
        }
        Value::Array(items) if !items.is_empty() => {
            out.push('\n');
            write_sequence(out, items, indent + INDENT_STEP);
        }
        other => {
            out.push(' ');
            out.push_str(&scalar(other));
            out.push('\n');
        }
    }
}

/// Render a leaf value (or an empty container) as a single-line YAML scalar.
fn scalar(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => scalar_string(s),
        // Only reached for empty containers; non-empty ones render as blocks.
        Value::Array(_) => "[]".to_string(),
        Value::Object(_) => "{}".to_string(),
    }
}

/// Quote a string only when a plain scalar would be misread by a YAML parser.
fn scalar_string(s: &str) -> String {
    if needs_double_quote(s) {
        double_quote(s)
    } else if is_plain_safe(s) {
        s.to_string()
    } else {
        single_quote(s)
    }
}

/// True when the string contains characters a single-quoted scalar cannot carry
/// verbatim (control characters, quotes, or backslashes).
fn needs_double_quote(s: &str) -> bool {
    s.chars().any(|c| c == '"' || c == '\\' || c.is_control())
}

/// True when the string can be emitted as a bare (unquoted) plain scalar.
///
/// Conservative by design: anything that *might* be reinterpreted is rejected
/// and falls back to a quoted form, which is always valid YAML.
fn is_plain_safe(s: &str) -> bool {
    if s.is_empty() || s != s.trim() || is_reserved(s) || looks_like_number(s) {
        return false;
    }
    // Characters that carry special meaning at the start of a plain scalar.
    const SPECIAL_START: &[char] = &[
        '-', '?', ':', ',', '[', ']', '{', '}', '#', '&', '*', '!', '|', '>', '\'', '"', '%', '@',
        '`', ' ',
    ];
    let Some(first) = s.chars().next() else {
        return false;
    };
    if SPECIAL_START.contains(&first) {
        return false;
    }
    // Sequences that introduce a comment (` #`) or a mapping (`: ` / trailing `:`).
    if s.contains(": ") || s.contains(" #") || s.ends_with(':') {
        return false;
    }
    // Restrict the body to an unambiguous, printable subset.
    s.chars().all(|c| {
        c.is_ascii_alphanumeric()
            || matches!(c, ' ' | '.' | '_' | '/' | '+' | '-' | '@' | '(' | ')')
    })
}

/// True for tokens a YAML 1.1 parser would coerce to a non-string type.
fn is_reserved(s: &str) -> bool {
    matches!(
        s,
        "null"
            | "Null"
            | "NULL"
            | "~"
            | "true"
            | "True"
            | "TRUE"
            | "false"
            | "False"
            | "FALSE"
            | "yes"
            | "Yes"
            | "YES"
            | "no"
            | "No"
            | "NO"
            | "on"
            | "On"
            | "ON"
            | "off"
            | "Off"
            | "OFF"
    )
}

/// True when the string would parse as a YAML number (and so must be quoted to
/// stay a string). IP addresses such as `8.8.8.8` are not numbers and pass.
fn looks_like_number(s: &str) -> bool {
    s.parse::<i64>().is_ok() || s.parse::<u64>().is_ok() || s.parse::<f64>().is_ok()
}

/// Emit a double-quoted scalar with C-style escapes for control characters.
fn double_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if c.is_control() => out.push_str(&format!("\\x{:02X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Emit a single-quoted scalar, doubling any embedded single quote.
fn single_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    out.push_str(&s.replace('\'', "''"));
    out.push('\'');
    out
}

/// Append `indent` spaces to `out`.
fn pad(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push(' ');
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn flat_mapping() {
        let yaml = to_yaml(&json!({"name": "flux", "version": "0.1.0"}));
        assert_eq!(yaml, "name: flux\nversion: 0.1.0\n");
    }

    #[test]
    fn scalars_are_quoted_when_ambiguous() {
        let yaml = to_yaml(&json!({
            "empty": "",
            "flag": "true",
            "note": "a: b",
            "port": "853",
            "ratio": "1.0",
        }));
        assert_eq!(
            yaml,
            "empty: ''\nflag: 'true'\nnote: 'a: b'\nport: '853'\nratio: '1.0'\n"
        );
    }

    #[test]
    fn ip_address_stays_plain() {
        // Dotted quads are not valid numbers and must not be quoted.
        let yaml = to_yaml(&json!({"ipv4": "8.8.8.8"}));
        assert_eq!(yaml, "ipv4: 8.8.8.8\n");
    }

    #[test]
    fn nested_array_of_objects() {
        let yaml = to_yaml(&json!({
            "data": [
                {"slug": "google", "protocols": ["plain", "dot"]},
                {"slug": "quad9"},
            ],
        }));
        let expected = "data:\n  - protocols:\n      - plain\n      - dot\n    slug: google\n  - slug: quad9\n";
        assert_eq!(yaml, expected);
    }

    #[test]
    fn empty_containers_render_inline() {
        let yaml = to_yaml(&json!({"a": {}, "b": []}));
        assert_eq!(yaml, "a: {}\nb: []\n");
    }

    #[test]
    fn numbers_bools_and_null() {
        let yaml = to_yaml(&json!({"count": 3, "ok": true, "missing": null}));
        // Keys sort alphabetically (serde_json default Map is ordered).
        assert_eq!(yaml, "count: 3\nmissing: null\nok: true\n");
    }

    #[test]
    fn top_level_scalar() {
        assert_eq!(to_yaml(&json!("hi")), "hi\n");
        assert_eq!(to_yaml(&json!(42)), "42\n");
        assert_eq!(to_yaml(&Value::Null), "null\n");
    }

    #[test]
    fn control_chars_force_double_quote() {
        let yaml = to_yaml(&json!({"x": "a\"b\nc"}));
        assert_eq!(yaml, "x: \"a\\\"b\\nc\"\n");
    }
}
