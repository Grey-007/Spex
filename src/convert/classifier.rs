use super::fuzzy::fuzzy_match;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticCategory {
    Role(String),
    Palette(usize),
    Unknown,
}

pub fn classify_token(token: &str) -> SemanticCategory {
    let normalized = normalize_name(token);

    if let Some(idx) = parse_palette_index(&normalized) {
        return SemanticCategory::Palette(idx);
    }

    if let Some(role) = alias_to_role(&normalized) {
        return SemanticCategory::Role(role.to_string());
    }

    classify_fuzzy(&normalized)
}

fn classify_fuzzy(normalized: &str) -> SemanticCategory {
    let role_candidates: [(&str, &[&str]); 26] = [
        ("background", &["background", "bg", "base_bg", "base"]),
        (
            "foreground",
            &["foreground", "fg", "text", "on_surface", "body_text"],
        ),
        ("primary", &["primary", "accent", "main", "highlight"]),
        (
            "primary_container",
            &["primary_container", "primary_bg", "accent_container"],
        ),
        ("on_primary", &["on_primary", "primary_text", "primary_fg"]),
        (
            "on_primary_container",
            &["on_primary_container", "primary_container_text"],
        ),
        ("secondary", &["secondary", "accent2", "sub", "alt"]),
        (
            "secondary_container",
            &["secondary_container", "secondary_bg", "sub_container"],
        ),
        (
            "on_secondary",
            &["on_secondary", "secondary_text", "secondary_fg"],
        ),
        (
            "on_secondary_container",
            &["on_secondary_container", "secondary_container_text"],
        ),
        ("tertiary", &["tertiary", "tertiary_accent"]),
        (
            "tertiary_container",
            &["tertiary_container", "tertiary_bg", "tertiary_surface"],
        ),
        ("on_tertiary", &["on_tertiary", "tertiary_text"]),
        (
            "on_tertiary_container",
            &["on_tertiary_container", "tertiary_container_text"],
        ),
        ("error", &["error", "danger", "critical", "alert"]),
        (
            "error_container",
            &["error_container", "danger_container", "error_bg"],
        ),
        ("on_error", &["on_error", "error_text"]),
        (
            "on_error_container",
            &["on_error_container", "error_container_text"],
        ),
        ("surface", &["surface", "panel", "card", "layer"]),
        (
            "surface_variant",
            &["surface_variant", "surface_alt", "surface_muted"],
        ),
        (
            "surface_container",
            &["surface_container", "surface_mid", "surface_level"],
        ),
        (
            "surface_container_low",
            &["surface_container_low", "surface_low"],
        ),
        (
            "surface_container_high",
            &["surface_container_high", "surface_high"],
        ),
        (
            "surface_container_highest",
            &["surface_container_highest", "surface_top"],
        ),
        ("outline", &["outline", "stroke"]),
        (
            "outline_variant",
            &["outline_variant", "outline_alt", "stroke_muted"],
        ),
    ];

    for (role, candidates) in role_candidates {
        if fuzzy_match(normalized, candidates, 4).is_some() {
            return SemanticCategory::Role(role.to_string());
        }
    }

    for (role, candidates) in [
        ("border", &["border", "divider", "separator"][..]),
        ("highlight", &["highlight", "focus", "hover"][..]),
        ("selection", &["selection", "selected", "active_bg"][..]),
    ] {
        if fuzzy_match(normalized, candidates, 5).is_some() {
            return SemanticCategory::Role(role.to_string());
        }
    }

    SemanticCategory::Unknown
}

fn normalize_name(input: &str) -> String {
    let mut value = input.trim().trim_start_matches('$').to_lowercase();
    value = value.replace('-', "_");

    if let Some(rest) = value.strip_prefix("colors.") {
        if let Some((role, _)) = rest.split_once(".default.") {
            return role.to_string();
        }

        if let Some((role, format)) = rest.rsplit_once('.') {
            if is_supported_format(format) {
                return role.to_string();
            }
        }

        return rest.to_string();
    }

    for suffix in [
        ".default.hex",
        ".default.rgb",
        ".default.rgba",
        ".default.hsl",
        ".hex",
        ".rgb",
        ".rgba",
        ".hsl",
    ] {
        if let Some(trimmed) = value.strip_suffix(suffix) {
            return trimmed.to_string();
        }
    }

    value
}

fn is_supported_format(format: &str) -> bool {
    matches!(format, "hex" | "rgb" | "rgba" | "hsl")
        || (format.starts_with("rgba(") && format.ends_with(')'))
}

fn alias_to_role(name: &str) -> Option<&'static str> {
    match name {
        "background" | "bg" | "base_bg" | "base" => Some("background"),
        "foreground" | "fg" | "text" | "on_surface" | "body_text" => Some("foreground"),

        "primary" | "accent" | "main" | "maincolor" => Some("primary"),
        "primary_container" | "primarycontainer" | "primary_bg" | "accent_container" => {
            Some("primary_container")
        }
        "on_primary" | "onprimary" | "primary_text" | "primary_fg" => Some("on_primary"),
        "on_primary_container" | "onprimarycontainer" | "primary_container_text" => {
            Some("on_primary_container")
        }

        "secondary" | "accent2" | "sub" | "alt" => Some("secondary"),
        "secondary_container" | "secondarycontainer" | "secondary_bg" | "sub_container" => {
            Some("secondary_container")
        }
        "on_secondary" | "onsecondary" | "secondary_text" | "secondary_fg" => Some("on_secondary"),
        "on_secondary_container" | "onsecondarycontainer" | "secondary_container_text" => {
            Some("on_secondary_container")
        }

        "tertiary" | "tertiary_accent" => Some("tertiary"),
        "tertiary_container" | "tertiarycontainer" | "tertiary_bg" | "tertiary_surface" => {
            Some("tertiary_container")
        }
        "on_tertiary" | "ontertiary" | "tertiary_text" => Some("on_tertiary"),
        "on_tertiary_container" | "ontertiarycontainer" | "tertiary_container_text" => {
            Some("on_tertiary_container")
        }

        "error" | "danger" | "critical" | "alert" | "error_color" => Some("error"),
        "error_container" | "errorcontainer" | "danger_container" | "error_bg" => {
            Some("error_container")
        }
        "on_error" | "onerror" | "error_text" => Some("on_error"),
        "on_error_container" | "onerrorcontainer" | "error_container_text" => {
            Some("on_error_container")
        }

        "surface" | "panel" | "card" | "layer" => Some("surface"),
        "surface_variant" | "surfacevariant" | "surface_alt" | "surface_muted"
        | "on_surface_variant" | "inverse_surface" | "surface_dim" | "surface_bright" => {
            Some("surface_variant")
        }
        "surface_container" | "surfacecontainer" | "surface_level" | "surface_mid" => {
            Some("surface_container")
        }
        "surface_container_low" | "surfacecontainerlow" | "surface_low" => {
            Some("surface_container_low")
        }
        "surface_container_high" | "surfacecontainerhigh" | "surface_high" => {
            Some("surface_container_high")
        }
        "surface_container_highest" | "surfacecontainerhighest" | "surface_top" => {
            Some("surface_container_highest")
        }

        "outline" | "stroke" => Some("outline"),
        "outline_variant" | "outlinevariant" | "outline_alt" | "stroke_muted" => {
            Some("outline_variant")
        }
        "border" | "divider" | "separator" => Some("border"),
        "highlight" | "focus" | "hover" => Some("highlight"),
        "selection" | "selected" | "active_bg" => Some("selection"),

        _ => None,
    }
}

fn parse_palette_index(normalized: &str) -> Option<usize> {
    for prefix in ["color", "palette"] {
        if let Some(index) = normalized.strip_prefix(prefix) {
            let idx = index.parse::<usize>().ok()?;
            if idx <= 255 {
                return Some(idx);
            }
        }
    }
    None
}
