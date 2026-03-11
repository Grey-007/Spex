use super::extractor::ExtractedToken;

#[derive(Debug, Clone, Copy)]
pub enum TemplateSystem {
    Pywal,
    Matugen,
    CssVariable,
    Unknown,
}

pub fn detect_template_system(tokens: &[ExtractedToken]) -> TemplateSystem {
    let has_pywal = has_any(tokens, &["color0", "background"]);
    let has_matugen = tokens
        .iter()
        .any(|t| t.normalized.starts_with("colors.") && t.normalized.contains(".default."));
    let has_css = tokens.iter().any(|t| {
        t.raw.starts_with('$')
            && matches!(
                t.normalized.as_str(),
                "background" | "foreground" | "accent" | "primary" | "secondary"
            )
    });

    if has_matugen {
        TemplateSystem::Matugen
    } else if has_pywal {
        TemplateSystem::Pywal
    } else if has_css {
        TemplateSystem::CssVariable
    } else {
        TemplateSystem::Unknown
    }
}

fn has_any(tokens: &[ExtractedToken], needles: &[&str]) -> bool {
    needles.iter().all(|needle| {
        tokens
            .iter()
            .any(|t| t.normalized.eq_ignore_ascii_case(needle))
    })
}
