use std::io;

use crate::palette::roles::ThemePalette;

pub fn export_terminal(theme: &ThemePalette) -> io::Result<()> {
    let content = format!(
        concat!(
            "background={}\n",
            "foreground={}\n",
            "color0={}\n",
            "color1={}\n",
            "color2={}\n",
            "color3={}\n",
            "color4={}\n",
            "color5={}\n",
            "color6={}\n",
            "color7={}\n"
        ),
        to_hex(theme.background),
        to_hex(theme.text),
        to_hex(theme.background),
        to_hex(theme.primary),
        to_hex(theme.secondary),
        to_hex(theme.accent),
        to_hex(theme.accent2),
        to_hex(theme.highlight),
        to_hex(theme.surface),
        to_hex(theme.text),
    );

    std::fs::write("spex.term", content)
}

fn to_hex(color: crate::models::color::Color) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b)
}
