use crate::palette::roles::ThemePalette;

use super::loops::render_color_loops;
use super::transform::resolve_token;

pub fn render(input: &str, palette: &ThemePalette) -> String {
    let looped = render_color_loops(input, &palette.colors);

    let mut output = String::with_capacity(looped.len());
    let mut cursor = 0usize;

    while let Some(relative_start) = looped[cursor..].find("{{") {
        let start = cursor + relative_start;
        output.push_str(&looped[cursor..start]);

        let Some(relative_end) = looped[start + 2..].find("}}") else {
            output.push_str(&looped[start..]);
            return output;
        };

        let end = start + 2 + relative_end;
        let token = &looped[start + 2..end];
        let original = &looped[start..end + 2];

        if let Some(replacement) = resolve_token(token.trim(), palette) {
            output.push_str(&replacement);
        } else {
            output.push_str(original);
        }

        cursor = end + 2;
    }

    output.push_str(&looped[cursor..]);
    output
}
