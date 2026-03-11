use std::collections::HashMap;

use super::extractor::ExtractedToken;

pub fn rewrite_template(
    input: &str,
    mappings: &HashMap<String, String>,
    extracted: &[ExtractedToken],
) -> String {
    let mut output = input.to_string();

    for token in extracted {
        if let Some(mapped) = mappings.get(&token.normalized) {
            output = output.replace(&token.raw, mapped);
        }
    }

    output
}
