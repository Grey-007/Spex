use crate::models::color::Color;

pub fn render_color_loops(input: &str, colors: &[Color]) -> String {
    let start_tag = "{{#colors}}";
    let end_tag = "{{/colors}}";

    let mut output = String::new();
    let mut cursor = 0usize;

    while let Some(relative_start) = input[cursor..].find(start_tag) {
        let start = cursor + relative_start;
        output.push_str(&input[cursor..start]);

        let block_start = start + start_tag.len();
        let Some(relative_end) = input[block_start..].find(end_tag) else {
            output.push_str(&input[start..]);
            return output;
        };

        let block_end = block_start + relative_end;
        let block = &input[block_start..block_end];

        for (index, color) in colors.iter().enumerate() {
            let row = block
                .replace("{{index}}", &index.to_string())
                .replace("{{value}}", &to_hex(*color));
            output.push_str(&row);
        }

        cursor = block_end + end_tag.len();
    }

    output.push_str(&input[cursor..]);
    output
}

fn to_hex(color: Color) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b)
}
