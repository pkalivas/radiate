use ratatui::style::{Color, Stylize, palette::material};

pub const BG_COLOR: Color = material::GRAY.c800;
pub const ALT_BG_COLOR: Color = material::GRAY.c900;
pub const TEXT_FG_COLOR: Color = material::GRAY.c300;

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

pub const SELECTED_GREEN: Color = material::GREEN.c300;

pub fn alternating_row_style(index: usize) -> ratatui::style::Style {
    if index % 2 == 0 {
        ratatui::style::Style::new().bg(BG_COLOR).fg(TEXT_FG_COLOR)
    } else {
        ratatui::style::Style::new()
            .bg(ALT_BG_COLOR)
            .fg(TEXT_FG_COLOR)
    }
}

pub fn selected_item_style() -> ratatui::style::Style {
    ratatui::style::Style::new()
        .fg(SELECTED_GREEN)
        .bg(material::BLACK)
        .reversed()
}
