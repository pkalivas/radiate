use ratatui::style::{Color, Style, Stylize, palette::material};

pub const NORMAL_ROW_BG: Color = material::GRAY.c800;
pub const ALT_ROW_BG_COLOR: Color = material::GRAY.c900;
pub const TEXT_FG_COLOR: Color = material::GRAY.c300;
pub const LIGHT_BLACK: Color = material::BLACK;

pub(crate) const COLOR_WHEEL_400: [Color; 8] = [
    material::RED.c400,
    material::BLUE.c400,
    material::GREEN.c400,
    material::YELLOW.c400,
    material::PURPLE.c400,
    material::CYAN.c400,
    material::ORANGE.c400,
    material::TEAL.c400,
];

pub(crate) const COLOR_WHEEL_600: [Color; 8] = [
    material::RED.c600,
    material::BLUE.c600,
    material::GREEN.c600,
    material::YELLOW.c600,
    material::PURPLE.c600,
    material::CYAN.c600,
    material::ORANGE.c600,
    material::TEAL.c600,
];

pub(crate) const COLOR_WHEEL_900: [Color; 8] = [
    material::RED.c900,
    material::BLUE.c900,
    material::GREEN.c900,
    material::YELLOW.c900,
    material::PURPLE.c900,
    material::CYAN.c900,
    material::ORANGE.c900,
    material::TEAL.c900,
];

pub const SELECTED_GREEN: Color = material::GREEN.c300;

pub fn alternating_row_style(index: usize) -> ratatui::style::Style {
    if index % 2 == 0 {
        ratatui::style::Style::new()
            .bg(NORMAL_ROW_BG)
            .fg(TEXT_FG_COLOR)
    } else {
        ratatui::style::Style::new()
            .bg(ALT_ROW_BG_COLOR)
            .fg(TEXT_FG_COLOR)
    }
}

pub fn selected_item_style() -> ratatui::style::Style {
    ratatui::style::Style::new()
        .fg(SELECTED_GREEN)
        .bg(material::BLACK)
        .reversed()
}
