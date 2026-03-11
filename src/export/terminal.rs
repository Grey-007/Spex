use std::io;
use std::path::PathBuf;

use crate::export::export_file_path;
use crate::export::template::render_template;
use crate::palette::roles::ThemePalette;

pub fn export_terminal(theme: &ThemePalette) -> io::Result<PathBuf> {
    let mut template = String::from(concat!(
        "background={{background}}\n",
        "foreground={{text}}\n",
        "surface={{surface}}\n",
        "primary={{primary}}\n",
        "secondary={{secondary}}\n",
        "accent={{accent}}\n",
        "accent2={{accent2}}\n",
        "highlight={{highlight}}\n"
    ));

    for (idx, _) in theme.colors.iter().enumerate() {
        template.push_str(&format!("color{idx}={{{{color{idx}}}}}\n"));
    }

    let content = render_template(&template, theme);

    let path = export_file_path("spex.term")?;
    std::fs::write(&path, content)?;
    Ok(path)
}
