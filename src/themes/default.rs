use floem::{
    peniko::Color,
    style::Style,
    taffy::{AlignContent, AlignItems},
};

pub const TEXT: Color = Color::BLACK;

pub fn body(style: Style) -> Style {
    style
        .font_size(16)
        .color(TEXT)
        .font_family("Bombardier".to_string())
}

pub fn secondary_button(style: Style) -> Style {
    body(style)
}

pub fn tag_grid_item(style: Style) -> Style {
    body(style)
        .width(50)
        .flex_grow(0.0)
        .max_width(50)
        .justify_center()
        .hover(|s| s.background(TEXT.multiply_alpha(40.0 / 255.0)))
}

pub fn h1(style: Style) -> Style {
    body(style).font_size(28)
}
pub fn h2(style: Style) -> Style {
    body(style).font_size(22)
}
pub fn h3(style: Style) -> Style {
    body(style).font_size(18)
}

pub fn table_item(style: Style) -> Style {
    style
        .border_bottom(1.0)
        .border_right(1.0)
        .border_color(TEXT)
        .align_self(AlignItems::Center)
        .align_content(AlignContent::Center)
        .justify_content(AlignContent::Center)
        .justify_self(AlignItems::Center)
        .width_full()
        .height_full()
        .min_height(22)
}
pub fn table_header(style: Style) -> Style {
    h3(style)
        .min_height(22)
        .border_bottom(1.0)
        .border_right(1.0)
        .border_color(TEXT)
}
pub fn grid(style: Style) -> Style {
    style.border_top(1.0).border_left(1.0).border_color(TEXT)
}

pub fn dropdown(style: Style) -> Style {
    style.border_radius(0.0)
}
