use std::collections::HashMap;

use regex::{Captures, Regex};

use super::extractor::ExtractedToken;

pub fn rewrite_template(
    input: &str,
    mappings: &HashMap<String, String>,
    extracted: &[ExtractedToken],
) -> String {
    let nested_matugen = Regex::new(r"\{+\s*(colors\.[A-Za-z0-9_\.]+)\s*\}+")
        .expect("nested Matugen regex must compile");
    let generic_tokens = Regex::new(
        r"\{\{\s*([^{}]+?)\s*\}\}|\$\{([^{}\n]+?)\}|\{([^{}\n]+?)\}|\$([A-Za-z_][A-Za-z0-9_.-]*)",
    )
    .expect("generic token regex must compile");

    let mut output = nested_matugen
        .replace_all(input, |caps: &Captures| {
            rewrite_inner_token(caps.get(1).map(|m| m.as_str()).unwrap_or_default(), mappings)
                .unwrap_or_else(|| caps.get(0).unwrap().as_str().to_string())
        })
        .into_owned();

    if !extracted.is_empty() {
        output = generic_tokens
            .replace_all(&output, |caps: &Captures| {
                let inner = first_capture(caps).unwrap_or_default();
                rewrite_inner_token(inner, mappings)
                    .unwrap_or_else(|| caps.get(0).unwrap().as_str().to_string())
            })
            .into_owned();
    }

    output
}

fn first_capture<'a>(caps: &'a Captures) -> Option<&'a str> {
    for idx in 1..=4 {
        if let Some(found) = caps.get(idx) {
            return Some(found.as_str().trim());
        }
    }
    None
}

fn rewrite_inner_token(token: &str, mappings: &HashMap<String, String>) -> Option<String> {
    if let Some(mapped) = mappings.get(token) {
        return Some(canonicalize_spex_token(mapped));
    }

    canonicalize_matugen_token(token).map(|inner| format!("{{{{{inner}}}}}"))
}

fn canonicalize_spex_token(mapped: &str) -> String {
    if let Some(inner) = mapped
        .trim()
        .strip_prefix("{{")
        .and_then(|m| m.strip_suffix("}}"))
    {
        if let Some(canonical) = canonicalize_matugen_token(inner.trim()) {
            return format!("{{{{{canonical}}}}}");
        }
    }

    mapped.to_string()
}

fn canonicalize_matugen_token(token: &str) -> Option<String> {
    let path = token.trim();
    let role = path
        .strip_prefix("colors.")?
        .strip_suffix(".default.hex")?
        .trim();
    if role.is_empty() {
        return None;
    }

    Some(format!("colors.{role}.hex"))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::rewrite_template;

    #[test]
    fn rewrites_triple_brace_matugen_token() {
        let input = "{{{colors.primary.default.hex}}}";
        let output = rewrite_template(input, &HashMap::new(), &[]);
        assert_eq!(output, "{{colors.primary.hex}}");
    }

    #[test]
    fn rewrites_deep_nested_matugen_token() {
        let input = "{{{{{{colors.outline.default.hex}}}}}}";
        let output = rewrite_template(input, &HashMap::new(), &[]);
        assert_eq!(output, "{{colors.outline.hex}}");
    }

    #[test]
    fn rewrites_nested_matugen_with_mapping() {
        let mut mappings = HashMap::new();
        mappings.insert(
            "colors.surface.default.hex".to_string(),
            "{{colors.surface.default.hex}}".to_string(),
        );
        let input = "{{{{colors.surface.default.hex}}}}";
        let output = rewrite_template(input, &mappings, &[]);
        assert_eq!(output, "{{colors.surface.hex}}");
    }
}
