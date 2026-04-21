//! Variable interpolation utilities.
//!
//! Doctag:templating

use std::collections::HashMap;

use regex::Regex;

use crate::core::models::KeyValue;

/// Interpolates `{{variable}}` placeholders from a context map.
pub fn interpolate(input: &str, context: &HashMap<String, String>) -> String {
    let pattern = Regex::new(r"\{\{\s*([a-zA-Z0-9_\-.]+)\s*\}\}").expect("valid regex");

    pattern
        .replace_all(input, |captures: &regex::Captures<'_>| {
            let key = captures
                .get(1)
                .map(|match_| match_.as_str())
                .unwrap_or_default();
            context
                .get(key)
                .cloned()
                .unwrap_or_else(|| captures[0].to_string())
        })
        .to_string()
}

/// Interpolates values for enabled key-value pairs and returns tuple vector.
pub fn interpolate_enabled_pairs(
    source: &[KeyValue],
    context: &HashMap<String, String>,
) -> Vec<(String, String)> {
    source
        .iter()
        .filter(|item| item.enabled)
        .map(|item| {
            (
                interpolate(&item.key, context),
                interpolate(&item.value, context),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::interpolate;

    #[test]
    fn resolves_placeholders() {
        let mut context = HashMap::new();
        context.insert("base".to_string(), "https://api.local".to_string());

        let output = interpolate("{{base}}/users", &context);
        assert_eq!(output, "https://api.local/users");
    }

    #[test]
    fn keeps_unknown_placeholder() {
        let context = HashMap::new();
        let output = interpolate("https://api/{{missing}}", &context);
        assert_eq!(output, "https://api/{{missing}}");
    }
}
