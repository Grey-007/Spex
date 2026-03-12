use super::classifier::SemanticCategory;

pub fn map_category(category: &SemanticCategory) -> Option<String> {
    match category {
        SemanticCategory::Role(role) => Some(format!("{{{{colors.{role}.default.hex}}}}")),
        SemanticCategory::Palette(index) => Some(format!("{{{{color{index}}}}}")),
        SemanticCategory::Unknown => None,
    }
}
