use super::fuzzy::fuzzy_match;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticCategory {
    Background,
    Foreground,
    Primary,
    Secondary,
    Palette(usize),
    Unknown,
}

pub fn classify_token(token: &str) -> SemanticCategory {
    let normalized = normalize_name(token);

    if let Some(idx) = parse_palette_index(&normalized) {
        return SemanticCategory::Palette(idx);
    }

    match normalized.as_str() {
        "background" | "bg" | "base" | "surface" | "base_bg" => SemanticCategory::Background,
        "foreground" | "fg" | "text" | "on_surface" => SemanticCategory::Foreground,
        "primary" | "accent" | "main" | "highlight" => SemanticCategory::Primary,
        "secondary" | "accent2" | "sub" => SemanticCategory::Secondary,
        _ => classify_fuzzy(&normalized),
    }
}

fn classify_fuzzy(normalized: &str) -> SemanticCategory {
    let bg_candidates = ["background", "bg", "surface", "base_bg"];
    let fg_candidates = ["foreground", "fg", "text", "on_surface"];
    let primary_candidates = ["primary", "accent", "main", "highlight", "maincolor"];
    let secondary_candidates = ["secondary", "accent2", "sub"];

    if fuzzy_match(normalized, &bg_candidates, 3).is_some() {
        return SemanticCategory::Background;
    }
    if fuzzy_match(normalized, &fg_candidates, 3).is_some() {
        return SemanticCategory::Foreground;
    }
    if fuzzy_match(normalized, &primary_candidates, 4).is_some() {
        return SemanticCategory::Primary;
    }
    if fuzzy_match(normalized, &secondary_candidates, 4).is_some() {
        return SemanticCategory::Secondary;
    }

    SemanticCategory::Unknown
}

fn normalize_name(input: &str) -> String {
    input
        .trim()
        .trim_start_matches("colors.")
        .trim_end_matches(".default.hex")
        .trim_end_matches(".default")
        .trim_end_matches(".hex")
        .to_lowercase()
}

fn parse_palette_index(normalized: &str) -> Option<usize> {
    if let Some(index) = normalized.strip_prefix("color") {
        let idx = index.parse::<usize>().ok()?;
        if idx <= 15 {
            return Some(idx);
        }
    }
    None
}
