use std::io;
use std::path::PathBuf;

use crate::export::export_file_path;
use crate::export::template::render_template;
use crate::palette::roles::ThemePalette;

pub fn export_css(theme: &ThemePalette) -> io::Result<PathBuf> {
    let mut template = String::from(
        concat!(
            ":root {\n",
            "  --background: {{background}};\n",
            "  --surface: {{surface}};\n",
            "  --primary: {{primary}};\n",
            "  --secondary: {{secondary}};\n",
            "  --accent: {{accent}};\n",
            "  --accent2: {{accent2}};\n",
            "  --highlight: {{highlight}};\n",
            "  --text: {{text}};\n"
        ),
    );

    for (idx, _) in theme.colors.iter().enumerate() {
        template.push_str(&format!("  --color{idx}: {{{{color{idx}}}}};\n"));
    }
    template.push_str("}\n");

    let content = render_template(&template, theme);

    let path = export_file_path("spex.css")?;
    std::fs::write(&path, content)?;
    Ok(path)
}
