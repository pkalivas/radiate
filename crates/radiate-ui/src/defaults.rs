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

pub(crate) const DISTRIBUTION_HEADER_CELLS: [&str; 11] = [
    "Metric", "Min", ".25p", ".50p", ".75p", "Max", "Count", "StdDev", "Var", "Skew", "Entr.",
];
