use ratatui::{
    style::{Color, Style, palette::material},
    widgets::{Block, Row},
};

pub const BG_COLOR: Color = material::GRAY.c800;
pub const ALT_BG_COLOR: Color = material::GRAY.c900;
pub const TEXT_FG_COLOR: Color = material::GRAY.c300;
pub const OVERLAY_COLOR: Color = material::GRAY.c700;

pub const TREND_UP_COLOR: Color = material::GREEN.c200;
pub const TREND_DOWN_COLOR: Color = material::RED.c200;
pub const TREND_FLAT_COLOR: Color = material::GRAY.c500;

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

/// Maps a 0–1 value to green / yellow / red (higher is better).
/// Pass `1.0 - value` for lower-is-better metrics.
pub fn sentiment_color(value: f32, warn: f32, good: f32) -> Color {
    if value >= good {
        TREND_UP_COLOR
    } else if value >= warn {
        material::YELLOW.c400
    } else {
        TREND_DOWN_COLOR
    }
}

/// Green if `last` is meaningfully above `mean`, red if below, gray if flat.
pub fn trend_color(last: f32, mean: f32) -> Color {
    let eps = mean.abs() * 0.01 + f32::EPSILON;
    if last > mean + eps {
        TREND_UP_COLOR
    } else if last < mean - eps {
        TREND_DOWN_COLOR
    } else {
        TREND_FLAT_COLOR
    }
}

/// ↑ / → / ↓ symbol for the last-vs-mean trend.
pub fn trend_symbol(last: f32, mean: f32) -> &'static str {
    let eps = mean.abs() * 0.01 + f32::EPSILON;
    if last > mean + eps {
        "↑"
    } else if last < mean - eps {
        "↓"
    } else {
        "→"
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
