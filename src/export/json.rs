use std::io;
use std::path::PathBuf;

use crate::export::export_file_path;
use crate::export::template::render_template;
use crate::palette::roles::ThemePalette;

pub fn export_json(theme: &ThemePalette) -> io::Result<PathBuf> {
    let mut template = String::from(
        concat!(
            "{\n",
            "  \"background\": \"{{background}}\",\n",
            "  \"surface\": \"{{surface}}\",\n",
            "  \"primary\": \"{{primary}}\",\n",
            "  \"secondary\": \"{{secondary}}\",\n",
            "  \"accent\": \"{{accent}}\",\n",
            "  \"accent2\": \"{{accent2}}\",\n",
            "  \"highlight\": \"{{highlight}}\",\n",
            "  \"text\": \"{{text}}\",\n",
            "  \"colors\": [\n"
        ),
    );

    for (idx, _) in theme.colors.iter().enumerate() {
        if idx > 0 {
            template.push_str(",\n");
        }
        template.push_str(&format!("    \"{{{{color{idx}}}}}\""));
    }
    template.push_str("\n  ]\n}\n");

    let content = render_template(&template, theme);

    let path = export_file_path("spex.json")?;
    std::fs::write(&path, content)?;
    Ok(path)
}
