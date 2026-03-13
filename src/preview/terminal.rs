use crate::models::color::Color;

const COLORS_PER_ROW: usize = 8;

pub fn print_palette(colors: &[Color]) {
    println!("Palette ({} colors, sorted by luminance)", colors.len());
    println!();

    for (idx, color) in colors.iter().enumerate() {
        print!("\x1b[48;2;{};{};{}m    \x1b[0m", color.r, color.g, color.b);
        if (idx + 1) % COLORS_PER_ROW == 0 {
            println!();
        }
    }

    if !colors.len().is_multiple_of(COLORS_PER_ROW) {
        println!();
    }

    println!();
    for color in colors {
        println!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b);
    }
}
