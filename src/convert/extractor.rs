use regex::Regex;

#[derive(Debug, Clone)]
pub struct ExtractedToken {
    pub raw: String,
    pub normalized: String,
    pub line: usize,
}

pub fn extract_tokens(input: &str) -> Result<Vec<ExtractedToken>, regex::Error> {
    let brace2 = Regex::new(r"\{\{\s*([^{}]+?)\s*\}\}")?;
    let brace1 = Regex::new(r"\{([^{}\n]+?)\}")?;
    let dollar_brace = Regex::new(r"\$\{([^{}\n]+?)\}")?;
    let dollar_word = Regex::new(r"\$([A-Za-z_][A-Za-z0-9_.-]*)")?;

    let mut out = Vec::new();
    for (idx, line) in input.lines().enumerate() {
        let line_no = idx + 1;
        collect_matches(&brace2, line, line_no, &mut out);
        collect_matches(&dollar_brace, line, line_no, &mut out);
        collect_matches(&brace1, line, line_no, &mut out);
        collect_matches(&dollar_word, line, line_no, &mut out);
    }

    out.sort_by(|a, b| a.line.cmp(&b.line).then(a.raw.cmp(&b.raw)));
    out.dedup_by(|a, b| a.raw == b.raw && a.line == b.line);
    Ok(out)
}

fn collect_matches(re: &Regex, line: &str, line_no: usize, out: &mut Vec<ExtractedToken>) {
    for captures in re.captures_iter(line) {
        let Some(full) = captures.get(0) else {
            continue;
        };

        let normalized = captures
            .get(1)
            .map(|v| v.as_str().trim().to_string())
            .unwrap_or_default();

        if normalized.is_empty() {
            continue;
        }

        out.push(ExtractedToken {
            raw: full.as_str().to_string(),
            normalized,
            line: line_no,
        });
    }
}
