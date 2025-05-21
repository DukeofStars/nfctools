use floem::{
    peniko::Color,
    style::Style,
    taffy::{AlignContent, AlignItems},
};

pub const TEXT: Color = Color::WHITE;
pub const BG: Color = Color::rgb8(8, 8, 8);
pub const BG2: Color = Color::rgb8(16, 16, 16);
pub const BG3: Color = Color::rgb8(22, 22, 22);
pub const BORDER: Color = Color::rgb8(99, 101, 99);
pub const HIGHLIGHT: Color = Color::rgb8(148, 59, 15);

pub fn body(style: Style) -> Style {
    style
        .font_size(16)
        .color(TEXT)
        .background(BG)
        .font_family("Bombardier".to_string())
}

pub fn list_item(style: Style) -> Style {
    style
        .background(BG2)
        .border_color(BORDER)
        .border(1.0)
        .border_bottom(0.0)
        .hover(|style| style.background(TEXT.multiply_alpha(40.0 / 255.0)))
}

pub fn main_menu_button(style: Style) -> Style {
    primary_button(h2(style))
        .width(200)
        .height(200)
        .border(2)
        .border_color(BORDER)
        .border_radius(4)
        .padding(10)
}

pub fn primary_button(style: Style) -> Style {
    body(style).color(HIGHLIGHT).border_color(HIGHLIGHT)
}
pub fn secondary_button(style: Style) -> Style {
    body(style).border_color(BORDER)
}

pub fn text_area(style: Style) -> Style {
    body(style)
        .border(3)
        .border_color(BORDER)
        .hover(|style| style.background(TEXT.multiply_alpha(0.05)))
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
        .background(BG3)
        .border_bottom(1.0)
        .border_right(1.0)
        .border_color(BORDER)
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
        .border_color(BORDER)
        .background(BG2)
}
pub fn grid(style: Style) -> Style {
    style.border_top(1.0).border_left(1.0).border_color(BORDER)
}

pub fn dropdown(style: Style) -> Style {
    style.border_radius(0.0)
}
pub fn dropdown_item_view(style: Style) -> Style {
    body(style).width_full().height_full()
}
