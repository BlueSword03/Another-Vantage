pub mod config;
pub mod firmware;
pub mod hardware;
pub mod model;
pub mod spec;

/// Parse a `charge_types` value such as `[Fast] Standard Long_Life`.
/// Returns the option list plus the currently active option.
pub fn parse_charge_types(s: Option<&str>) -> (Vec<String>, Option<String>) {
    let mut options = Vec::new();
    let mut active = None;
    if let Some(s) = s {
        for tok in s.split_whitespace() {
            if let Some(stripped) = tok.strip_prefix('[').and_then(|t| t.strip_suffix(']')) {
                active = Some(stripped.to_string());
                options.push(stripped.to_string());
            } else {
                options.push(tok.to_string());
            }
        }
    }
    (options, active)
}

#[cfg(test)]
mod tests {
    use super::parse_charge_types;

    #[test]
    fn parses_active_marker() {
        let (options, active) = parse_charge_types(Some("[Fast] Standard Long_Life"));
        assert_eq!(options, vec!["Fast", "Standard", "Long_Life"]);
        assert_eq!(active.as_deref(), Some("Fast"));
    }

    #[test]
    fn parses_without_marker() {
        let (options, active) = parse_charge_types(Some("Standard Long_Life"));
        assert_eq!(options, vec!["Standard", "Long_Life"]);
        assert_eq!(active, None);
    }

    #[test]
    fn handles_missing_value() {
        let (options, active) = parse_charge_types(None);
        assert!(options.is_empty());
        assert_eq!(active, None);
    }
}
