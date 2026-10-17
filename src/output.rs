use serde_json::{Map, Value};

/// Human-readable rendering of the section values. Field objects of the shape
/// `{value, reason?, hint?}` are rendered inline with their note.
pub fn render(sections: &[(String, Value)], color: bool) -> String {
    let (bold, reset) = if color {
        ("\x1b[1m", "\x1b[0m")
    } else {
        ("", "")
    };
    let mut out = String::new();
    for (name, v) in sections {
        out.push_str(&format!("\n{bold}[{name}]{reset}\n"));
        render_object(v, 0, &mut out);
    }
    out
}

fn render_object(v: &Value, indent: usize, out: &mut String) {
    match v {
        Value::Object(map) => {
            for (k, val) in map {
                render_entry(k, val, indent, out);
            }
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                out.push_str(&format!("{}[]\n", pad(indent)));
            } else {
                for x in arr {
                    render_bullet(x, indent, out);
                }
            }
        }
        other => out.push_str(&format!("{}{}\n", pad(indent), scalar(other))),
    }
}

fn render_entry(key: &str, v: &Value, indent: usize, out: &mut String) {
    let p = pad(indent);
    match v {
        Value::Object(map) if is_field(map) => {
            out.push_str(&format!("{p}{key}: {}\n", field_text(map)));
        }
        Value::Object(_) | Value::Array(_) => {
            out.push_str(&format!("{p}{key}:\n"));
            render_object(v, indent + 1, out);
        }
        other => out.push_str(&format!("{p}{key}: {}\n", scalar(other))),
    }
}

fn render_bullet(v: &Value, indent: usize, out: &mut String) {
    let p = pad(indent);
    match v {
        Value::Object(map) if is_field(map) => {
            out.push_str(&format!("{p}- {}\n", field_text(map)));
        }
        Value::Object(map) => {
            let mut iter = map.iter();
            match iter.next() {
                Some((k, v)) if !matches!(v, Value::Object(_) | Value::Array(_)) => {
                    out.push_str(&format!("{p}- {k}: {}\n", scalar(v)));
                }
                Some((k, v)) => {
                    out.push_str(&format!("{p}-\n"));
                    render_entry(k, v, indent + 1, out);
                }
                None => out.push_str(&format!("{p}- {{}}\n")),
            }
            for (k, v) in iter {
                render_entry(k, v, indent + 1, out);
            }
        }
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(scalar).collect();
            out.push_str(&format!("{p}- {}\n", items.join(", ")));
        }
        other => out.push_str(&format!("{p}- {}\n", scalar(other))),
    }
}

fn is_field(map: &Map<String, Value>) -> bool {
    !map.is_empty()
        && map
            .keys()
            .all(|k| matches!(k.as_str(), "value" | "reason" | "hint"))
}

fn field_text(map: &Map<String, Value>) -> String {
    let value = map.get("value").unwrap_or(&Value::Null);
    let mut s = if value.is_null() {
        "不可用".to_string()
    } else {
        scalar(value)
    };
    if let Some(reason) = map.get("reason").and_then(|v| v.as_str()) {
        s.push_str(&format!("  # {reason}"));
    }
    if let Some(hint) = map.get("hint").and_then(|v| v.as_str()) {
        s.push_str(&format!(" ({hint})"));
    }
    s
}

fn scalar(v: &Value) -> String {
    match v {
        Value::Null => "未知".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn pad(n: usize) -> String {
    "  ".repeat(n)
}
