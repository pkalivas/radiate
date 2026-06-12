use ratatui::{
    style::{Color, Style, palette::material},
    widgets::{Block, Row},
};

pub const BG_COLOR: Color = material::GRAY.c800;
pub const ALT_BG_COLOR: Color = material::GRAY.c900;
pub const TEXT_FG_COLOR: Color = material::GRAY.c300;
pub const OVERLAY_COLOR: Color = material::GRAY.c700;

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
pub const BORDER_GREEN: Color = material::GREEN.c400;

pub fn alternating_row_style(index: usize) -> ratatui::style::Style {
    if index.is_multiple_of(2) {
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

pub fn panel_block(focused: bool) -> Block<'static> {
    let base = tui_piechart::border_style::BorderStyle::Rounded.block();
    if focused {
        base.border_style(Style::default().fg(BORDER_GREEN))
    } else {
        base
    }
}

pub fn striped_rows<'a>(rows: impl IntoIterator<Item = Row<'a>>) -> impl Iterator<Item = Row<'a>> {
    rows.into_iter().enumerate().map(|(i, row)| {
        let bg = if i.is_multiple_of(2) {
            BG_COLOR
        } else {
            ALT_BG_COLOR
        };
        row.style(Style::default().bg(bg))
    })
}
