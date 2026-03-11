use std::io;

use crate::palette::roles::ThemePalette;

pub fn export_css(theme: &ThemePalette) -> io::Result<()> {
    let content = format!(
        concat!(
            ":root {{\n",
            "  --background: {};\n",
            "  --surface: {};\n",
            "  --primary: {};\n",
            "  --secondary: {};\n",
            "  --accent: {};\n",
            "  --accent2: {};\n",
            "  --highlight: {};\n",
            "  --text: {};\n",
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

    std::fs::write("spex.css", content)
}

fn to_hex(color: crate::models::color::Color) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b)
}
