use crate::models::color::Color;
use crate::palette::roles::ThemePalette;

pub fn render_template(template: &str, palette: &ThemePalette) -> String {
    let mut rendered = template.to_string();

    for (name, color) in semantic_roles(palette) {
        let placeholder = format!("{{{{{name}}}}}");
        rendered = rendered.replace(&placeholder, &to_hex(color));
    }

    render_dynamic_colors(&rendered, palette)
}

fn semantic_roles(palette: &ThemePalette) -> [(&'static str, Color); 8] {
    [
        ("background", palette.background),
        ("surface", palette.surface),
        ("primary", palette.primary),
        ("secondary", palette.secondary),
        ("accent", palette.accent),
        ("accent2", palette.accent2),
        ("highlight", palette.highlight),
        ("text", palette.text),
    ]
}

fn render_dynamic_colors(input: &str, palette: &ThemePalette) -> String {
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0usize;

    while let Some(relative_start) = input[cursor..].find("{{color") {
        let start = cursor + relative_start;
        output.push_str(&input[cursor..start]);

        let Some(relative_end) = input[start..].find("}}") else {
            output.push_str(&input[start..]);
            return output;
        };

        let end = start + relative_end + 2;
        let token = &input[start..end];
        let index_text = &input[start + 7..end - 2];

        if let Ok(index) = index_text.parse::<usize>() {
            if let Some(color) = palette.colors.get(index) {
                output.push_str(&to_hex(*color));
            } else {
                output.push_str(token);
            }
        } else {
            output.push_str(token);
        }

        cursor = end;
    }

    output.push_str(&input[cursor..]);
    output
}

fn to_hex(color: Color) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b)
}
