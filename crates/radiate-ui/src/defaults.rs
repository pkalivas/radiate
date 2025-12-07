use ratatui::style::{Color, palette::material};

pub const NORMAL_ROW_BG: Color = material::GRAY.c800;
pub const ALT_ROW_BG_COLOR: Color = material::GRAY.c900;
pub const TEXT_FG_COLOR: Color = material::GRAY.c300;

pub const STAT_HEADER_CELLS: [&str; 8] = [
    "Metric",
    "Min",
    "Max",
    "μ (mean)",
    "Sum",
    "StdDev",
    "Var",
    "Count",
];

pub const TIME_HEADER_CELLS: [&str; 5] = ["Metric", "Min", "Max", "μ (mean)", "Total"];
pub(crate) const ENGINE_HEADER_CELLS: [&str; 2] = ["Metric", "Val"];
pub(crate) const DISTRIBUTION_HEADER_CELLS: [&str; 11] = [
    "Metric", "Min", ".25p", ".50p", ".75p", "Max", "Count", "StdDev", "Var", "Skew", "Entr.",
];

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
