use std::io;
use std::path::PathBuf;

use crate::export::export_file_path;
use crate::palette::roles::ThemePalette;

pub fn export_json(theme: &ThemePalette) -> io::Result<PathBuf> {
    let content = format!(
        concat!(
            "{{\n",
            "  \"background\": \"{}\",\n",
            "  \"surface\": \"{}\",\n",
            "  \"primary\": \"{}\",\n",
            "  \"secondary\": \"{}\",\n",
            "  \"accent\": \"{}\",\n",
            "  \"accent2\": \"{}\",\n",
            "  \"highlight\": \"{}\",\n",
            "  \"text\": \"{}\"\n",
            "}}\n"
        ),
        to_hex(theme.background),
        to_hex(theme.surface),
        to_hex(theme.primary),
        to_hex(theme.secondary),
        to_hex(theme.accent),
        to_hex(theme.accent2),
        to_hex(theme.highlight),
        to_hex(theme.text),
    );

    let path = export_file_path("spex.json")?;
    std::fs::write(&path, content)?;
    Ok(path)
}

fn to_hex(color: crate::models::color::Color) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b)
}
