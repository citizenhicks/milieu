use std::env;

#[derive(Clone, Copy)]
pub struct Rgb(pub u8, pub u8, pub u8);

pub const MAUVE: Rgb = Rgb(203, 166, 247);
pub const RED: Rgb = Rgb(243, 139, 168);
pub const PEACH: Rgb = Rgb(250, 179, 135);
pub const YELLOW: Rgb = Rgb(249, 226, 175);
pub const GREEN: Rgb = Rgb(166, 227, 161);
pub const SKY: Rgb = Rgb(137, 220, 235);
pub const LAVENDER: Rgb = Rgb(180, 190, 254);
pub const TEXT: Rgb = Rgb(205, 214, 244);
pub const SUBTEXT1: Rgb = Rgb(186, 194, 222);

pub fn enabled() -> bool {
    if env::var_os("NO_COLOR").is_some() {
        return false;
    }
    atty::is(atty::Stream::Stdout)
}

pub fn paint(color: Rgb, text: &str) -> String {
    if !enabled() {
        return text.to_string();
    }
    format!("\x1b[38;2;{};{};{}m{}\x1b[0m", color.0, color.1, color.2, text)
}

pub fn bold(color: Rgb, text: &str) -> String {
    if !enabled() {
        return text.to_string();
    }
    format!(
        "\x1b[1;38;2;{};{};{}m{}\x1b[0m",
        color.0, color.1, color.2, text
    )
}
