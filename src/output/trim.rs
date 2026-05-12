// SPDX-License-Identifier: GPL-3.0-or-later

use serde_json::{Map, Value};

/// Trim a JSON value to contain only the requested field paths.
///
/// * `value` – the JSON value to filter (typically the `data` payload).
/// * `fields` – list of field path strings; dot notation is supported for
///   nested access (e.g. `metadata.version`).
///
/// Returns the filtered value.  Non-existent fields are silently omitted.
/// If `fields` is empty or `value` is not an object the value is returned
/// unchanged.
pub fn trim_value(value: Value, fields: &[String]) -> Value {
    if fields.is_empty() {
        return value;
    }
    if !value.is_object() {
        return value;
    }

    let mut result = Map::new();
    for field in fields {
        let path: Vec<&str> = field.split('.').collect();
        if let Some(v) = get_nested(&value, &path) {
            set_nested(&mut result, &path, v);
        }
    }
    Value::Object(result)
}

fn get_nested(value: &Value, path: &[&str]) -> Option<Value> {
    match path.split_first() {
        None => Some(value.clone()),
        Some((head, tail)) => {
            let obj = value.as_object()?;
            let next = obj.get(*head)?;
            get_nested(next, tail)
        }
    }
}

fn set_nested(obj: &mut Map<String, Value>, path: &[&str], value: Value) {
    match path.split_first() {
        None => {}
        Some((head, [])) => {
            obj.insert(head.to_string(), value);
        }
        Some((head, tail)) => {
            let entry = obj
                .entry(head.to_string())
                .or_insert_with(|| Value::Object(Map::new()));
            if let Value::Object(inner) = entry {
                set_nested(inner, tail, value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obj(pairs: &[(&str, Value)]) -> Value {
        let mut m = Map::new();
        for (k, v) in pairs {
            m.insert(k.to_string(), v.clone());
        }
        Value::Object(m)
    }

    #[test]
    fn test_basic_field_selection() {
        let value = obj(&[
            ("slug", Value::String("cloudflare".into())),
            ("name", Value::String("Cloudflare".into())),
        ]);
        let trimmed = trim_value(value, &["slug".to_string()]);
        let expected = obj(&[("slug", Value::String("cloudflare".into()))]);
        assert_eq!(trimmed, expected);
    }

    #[test]
    fn test_multiple_fields() {
        let value = obj(&[
            ("slug", Value::String("cloudflare".into())),
            ("name", Value::String("Cloudflare".into())),
            (
                "protocols",
                Value::Array(vec![
                    Value::String("DoH".into()),
                    Value::String("DoT".into()),
                ]),
            ),
        ]);
        let trimmed = trim_value(value, &["slug".to_string(), "protocols".to_string()]);
        let expected = obj(&[
            ("slug", Value::String("cloudflare".into())),
            (
                "protocols",
                Value::Array(vec![
                    Value::String("DoH".into()),
                    Value::String("DoT".into()),
                ]),
            ),
        ]);
        assert_eq!(trimmed, expected);
    }

    #[test]
    fn test_nonexistent_field_omitted() {
        let value = obj(&[("slug", Value::String("cloudflare".into()))]);
        let trimmed = trim_value(value, &["slug".to_string(), "missing".to_string()]);
        let expected = obj(&[("slug", Value::String("cloudflare".into()))]);
        assert_eq!(trimmed, expected);
    }

    #[test]
    fn test_empty_fields_list_returns_unchanged() {
        let value = obj(&[("slug", Value::String("cloudflare".into()))]);
        let trimmed = trim_value(value.clone(), &[]);
        assert_eq!(trimmed, value);
    }

    #[test]
    fn test_nested_object_trimming() {
        let inner = obj(&[
            ("tool", Value::String("flux".into())),
            ("version", Value::String("0.1.0".into())),
        ]);
        let value = obj(&[
            ("slug", Value::String("cloudflare".into())),
            ("metadata", inner),
        ]);
        let trimmed = trim_value(value, &["metadata.tool".to_string()]);
        let expected = obj(&[("metadata", obj(&[("tool", Value::String("flux".into()))]))]);
        assert_eq!(trimmed, expected);
    }

    #[test]
    fn test_multiple_nested_fields() {
        let inner = obj(&[
            ("tool", Value::String("flux".into())),
            ("version", Value::String("0.1.0".into())),
        ]);
        let value = obj(&[
            ("slug", Value::String("cloudflare".into())),
            ("metadata", inner),
        ]);
        let trimmed = trim_value(
            value,
            &["metadata.tool".to_string(), "metadata.version".to_string()],
        );
        let expected = obj(&[(
            "metadata",
            obj(&[
                ("tool", Value::String("flux".into())),
                ("version", Value::String("0.1.0".into())),
            ]),
        )]);
        assert_eq!(trimmed, expected);
    }

    #[test]
    fn test_non_object_returns_unchanged() {
        let value = Value::String("just a string".into());
        let trimmed = trim_value(value.clone(), &["slug".to_string()]);
        assert_eq!(trimmed, value);
    }

    #[test]
    fn test_deeply_nested_path() {
        let deep = obj(&[("value", Value::Number(42.into()))]);
        let mid = obj(&[("deep", deep)]);
        let value = obj(&[("mid", mid)]);
        let trimmed = trim_value(value, &["mid.deep.value".to_string()]);
        let expected = obj(&[(
            "mid",
            obj(&[("deep", obj(&[("value", Value::Number(42.into()))]))]),
        )]);
        assert_eq!(trimmed, expected);
    }
}
