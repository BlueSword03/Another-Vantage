use serde_json::{Map, Value};

use crate::i18n::{bool_text, charge_mode, label, t, Msg};

/// Marks values that get a localized display name (charge modes only for now;
/// performance profile values are deliberately left raw because their Chinese
/// names differ per model).
const CHARGE_CTX: &str = "charge_types";

/// Human-readable rendering of the section values. Field objects of the shape
/// `{value, reason?, hint?}` are rendered inline with their note. Keys and
/// values are localized for display only; `--json` is unaffected.
pub fn render(sections: &[(String, Value)], color: bool) -> String {
    let (bold, reset) = if color {
        ("\x1b[1m", "\x1b[0m")
    } else {
        ("", "")
    };
    let mut out = String::new();
    for (name, v) in sections {
        out.push_str(&format!("\n{bold}[{name}]{reset}\n"));
        render_object(v, 0, &mut out, None);
    }
    out
}

fn render_object(v: &Value, indent: usize, out: &mut String, ctx: Option<&str>) {
    match v {
        Value::Object(map) => {
            for (k, val) in map {
                render_entry(k, val, indent, out, ctx);
            }
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                out.push_str(&format!("{}[]\n", pad(indent)));
            } else {
                for x in arr {
                    render_bullet(x, indent, out, ctx);
                }
            }
        }
        other => out.push_str(&format!("{}{}\n", pad(indent), scalar(other, ctx))),
    }
}

fn render_entry(key: &str, v: &Value, indent: usize, out: &mut String, ctx: Option<&str>) {
    let p = pad(indent);
    let child_ctx = if key == CHARGE_CTX {
        Some(CHARGE_CTX)
    } else {
        ctx
    };
    let key = label(key);
    match v {
        Value::Object(map) if is_field(map) => {
            out.push_str(&format!("{p}{key}: {}\n", field_text(map)));
        }
        Value::Object(_) | Value::Array(_) => {
            out.push_str(&format!("{p}{key}:\n"));
            render_object(v, indent + 1, out, child_ctx);
        }
        other => out.push_str(&format!("{p}{key}: {}\n", scalar(other, child_ctx))),
    }
}

fn render_bullet(v: &Value, indent: usize, out: &mut String, ctx: Option<&str>) {
    let p = pad(indent);
    match v {
        Value::Object(map) if is_field(map) => {
            out.push_str(&format!("{p}- {}\n", field_text(map)));
        }
        Value::Object(map) => {
            let mut iter = map.iter();
            match iter.next() {
                Some((k, v)) if !matches!(v, Value::Object(_) | Value::Array(_)) => {
                    out.push_str(&format!("{p}- {}: {}\n", label(k), scalar(v, ctx)));
                }
                Some((k, v)) => {
                    out.push_str(&format!("{p}-\n"));
                    render_entry(k, v, indent + 1, out, ctx);
                }
                None => out.push_str(&format!("{p}- {{}}\n")),
            }
            for (k, v) in iter {
                render_entry(k, v, indent + 1, out, ctx);
            }
        }
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(|x| scalar(x, ctx)).collect();
            out.push_str(&format!("{p}- {}\n", items.join(", ")));
        }
        other => out.push_str(&format!("{p}- {}\n", scalar(other, ctx))),
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
        t(Msg::Unavailable).to_string()
    } else {
        scalar(value, None)
    };
    if let Some(reason) = map.get("reason").and_then(|v| v.as_str()) {
        s.push_str(&format!("  # {reason}"));
    }
    if let Some(hint) = map.get("hint").and_then(|v| v.as_str()) {
        s.push_str(&format!(" ({hint})"));
    }
    s
}

fn scalar(v: &Value, ctx: Option<&str>) -> String {
    if ctx == Some(CHARGE_CTX) {
        if let Value::String(s) = v {
            return charge_mode(s).to_string();
        }
    }
    match v {
        Value::Null => t(Msg::Unknown).into(),
        Value::Bool(b) => bool_text(*b).into(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn pad(n: usize) -> String {
    "  ".repeat(n)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::render;
    use crate::i18n::{init, Lang};

    #[test]
    fn localizes_charge_modes_but_not_performance_values() {
        init(Lang::Zh);
        let charge = render(
            &[(
                "charge".to_string(),
                json!({"charge_types": {"options": ["Fast", "Standard", "Long_Life"], "active": "Fast"}}),
            )],
            false,
        );
        assert!(charge.contains("快充"), "{charge}");
        assert!(charge.contains("常规"), "{charge}");
        assert!(charge.contains("养护"), "{charge}");
        assert!(!charge.contains("Fast"), "{charge}");
        assert!(!charge.contains("Long_Life"), "{charge}");

        // Performance profile values stay as the kernel reports them.
        let perf = render(
            &[(
                "performance".to_string(),
                json!({"platform_profile": "performance", "choices": ["low-power", "balanced", "performance"]}),
            )],
            false,
        );
        assert!(perf.contains("performance"), "{perf}");
        assert!(perf.contains("low-power"), "{perf}");

        init(Lang::En);
        let en = render(
            &[(
                "charge".to_string(),
                json!({"charge_types": {"options": ["Fast"], "active": "Fast"}}),
            )],
            false,
        );
        assert!(en.contains("Fast"), "{en}");
    }
}
