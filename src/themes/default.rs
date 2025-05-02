use floem::{peniko::Color, style::Style};

pub const TEXT: Color = Color::BLACK;

pub fn body(style: Style) -> Style {
    style.font_size(14).color(TEXT)
}

pub fn h1(style: Style) -> Style {
    body(style).font_size(24)
}
pub fn h2(style: Style) -> Style {
    body(style).font_size(18)
}
pub fn h3(style: Style) -> Style {
    body(style).font_size(16)
}
