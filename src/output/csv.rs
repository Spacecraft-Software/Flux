// SPDX-License-Identifier: GPL-3.0-or-later
// Rust guideline compliant 2026-05-18

//! Minimal RFC 4180-style CSV serializer for [`serde_json::Value`] payloads.
//!
//! CSV is a flat, tabular format, so Flux renders only the `data` payload here —
//! the `{ metadata, … }` envelope is JSON/YAML-specific and has no tabular
//! shape. Nested values within a cell are encoded as compact JSON to stay
//! lossless and unambiguous. Fields are LF-terminated (rather than RFC 4180's
//! CRLF) to match the rest of the tool's POSIX-friendly output.

use serde_json::{Map, Value};

/// Render a JSON value as CSV.
///
/// * An array of objects becomes a table whose header is the union of all keys.
/// * An array of scalars becomes a single `value` column.
/// * A single-key object wrapping an array (the shape every `list` command
///   returns, e.g. `{"providers": [..]}`) is unwrapped into that table.
/// * Any other object becomes a one-row table (keys as header).
/// * A bare scalar becomes a single cell.
/// * An empty array yields the empty string.
pub fn to_csv(value: &Value) -> String {
    match value {
        Value::Array(items) => array_to_csv(items),
        Value::Object(map) => {
            if let Some(items) = single_array_value(map) {
                return array_to_csv(items);
            }
            let headers: Vec<String> = map.keys().cloned().collect();
            let row: Vec<String> = map.values().map(cell).collect();
            let mut out = String::new();
            write_row(&mut out, &headers);
            write_row(&mut out, &row);
            out
        }
        other => {
            let mut out = String::new();
            write_row(&mut out, &[cell(other)]);
            out
        }
    }
}

/// If `map` has exactly one entry whose value is an array, return that array.
///
/// `list` payloads wrap their rows under a single key; unwrapping it lets CSV
/// render a real table instead of one opaque, fully-JSON-encoded cell.
fn single_array_value(map: &Map<String, Value>) -> Option<&[Value]> {
    if map.len() != 1 {
        return None;
    }
    match map.values().next() {
        Some(Value::Array(items)) => Some(items),
        _ => None,
    }
}

/// Render an array as CSV, choosing a tabular or single-column layout.
fn array_to_csv(items: &[Value]) -> String {
    if items.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    if items.iter().all(Value::is_object) {
        let headers = union_keys(items);
        write_row(&mut out, &headers);
        for item in items {
            let row: Vec<String> = headers
                .iter()
                .map(|h| {
                    item.as_object()
                        .and_then(|o| o.get(h))
                        .map_or(String::new(), cell)
                })
                .collect();
            write_row(&mut out, &row);
        }
    } else {
        // Heterogeneous or scalar array: one value per row under a single column.
        write_row(&mut out, &["value".to_string()]);
        for item in items {
            write_row(&mut out, &[cell(item)]);
        }
    }
    out
}

/// Collect the union of object keys across `items`, preserving first-seen order.
fn union_keys(items: &[Value]) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    for item in items {
        if let Some(obj) = item.as_object() {
            for k in obj.keys() {
                if !keys.iter().any(|existing| existing == k) {
                    keys.push(k.clone());
                }
            }
        }
    }
    keys
}

/// Convert a single JSON value to its CSV cell text.
fn cell(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        // Nested structures are encoded as compact JSON to remain lossless.
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

/// Append one CSV record (comma-separated, LF-terminated) to `out`.
fn write_row(out: &mut String, fields: &[String]) {
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&escape(field));
    }
    out.push('\n');
}

/// Quote a field per RFC 4180 when it contains a comma, quote, or line break.
fn escape(field: &str) -> String {
    if field.contains(['"', ',', '\n', '\r']) {
        let mut out = String::with_capacity(field.len() + 2);
        out.push('"');
        out.push_str(&field.replace('"', "\"\""));
        out.push('"');
        out
    } else {
        field.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn array_of_objects_is_tabular() {
        let csv = to_csv(&json!([
            {"slug": "google", "name": "Google"},
            {"slug": "quad9", "name": "Quad9"},
        ]));
        assert_eq!(csv, "name,slug\nGoogle,google\nQuad9,quad9\n");
    }

    #[test]
    fn missing_key_and_nested_cell() {
        let csv = to_csv(&json!([
            {"a": "x,y", "b": [1, 2]},
            {"a": "hi"},
        ]));
        // "x,y" is quoted; the nested array becomes compact JSON (itself quoted
        // because it contains a comma); the missing b cell is blank.
        assert_eq!(csv, "a,b\n\"x,y\",\"[1,2]\"\nhi,\n");
    }

    #[test]
    fn array_of_scalars_single_column() {
        let csv = to_csv(&json!(["plain", "dot"]));
        assert_eq!(csv, "value\nplain\ndot\n");
    }

    #[test]
    fn single_object_one_row() {
        let csv = to_csv(&json!({"slug": "google", "name": "Google"}));
        assert_eq!(csv, "name,slug\nGoogle,google\n");
    }

    #[test]
    fn bare_scalar_single_cell() {
        assert_eq!(to_csv(&json!("hi")), "hi\n");
        assert_eq!(to_csv(&json!(42)), "42\n");
    }

    #[test]
    fn empty_array_is_empty() {
        assert_eq!(to_csv(&json!([])), "");
    }

    #[test]
    fn embedded_quotes_are_doubled() {
        let csv = to_csv(&json!([{"k": "a\"b"}]));
        assert_eq!(csv, "k\n\"a\"\"b\"\n");
    }

    #[test]
    fn single_key_array_wrapper_unwraps_to_table() {
        // The shape every `dns list` subcommand returns.
        let csv = to_csv(&json!({
            "providers": [
                {"slug": "google", "name": "Google"},
                {"slug": "quad9", "name": "Quad9"},
            ],
        }));
        assert_eq!(csv, "name,slug\nGoogle,google\nQuad9,quad9\n");
    }

    #[test]
    fn multi_key_object_stays_one_row() {
        // A status-style object must not be mistaken for a list wrapper.
        let csv = to_csv(&json!({"backend": "systemd", "provider": "cloudflare"}));
        assert_eq!(csv, "backend,provider\nsystemd,cloudflare\n");
    }

    #[test]
    fn null_cell_is_blank() {
        let csv = to_csv(&json!([{"a": null, "b": "x"}]));
        assert_eq!(csv, "a,b\n,x\n");
    }
}
