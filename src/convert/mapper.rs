use super::classifier::SemanticCategory;

pub fn map_category(category: &SemanticCategory) -> Option<String> {
    match category {
        SemanticCategory::Background => Some("{{colors.background.default.hex}}".to_string()),
        SemanticCategory::Foreground => Some("{{colors.foreground.default.hex}}".to_string()),
        SemanticCategory::Primary => Some("{{colors.primary.default.hex}}".to_string()),
        SemanticCategory::Secondary => Some("{{colors.secondary.default.hex}}".to_string()),
        SemanticCategory::Palette(index) => Some(format!("{{{{color{index}}}}}")),
        SemanticCategory::Unknown => None,
    }
}
