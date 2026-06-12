use crate::state::{AppState, Pane};
use crate::widgets::AppWidget;
use radiate_engines::Chromosome;
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::{
    style::{Color, Style},
    widgets::{RenderDirection, Sparkline, SparklineBar, Widget},
};

pub struct SpeciesSparklineComponent;

impl SpeciesSparklineComponent {
    pub fn new() -> Self {
        Self
    }
}

impl<C: Chromosome> AppWidget<C> for SpeciesSparklineComponent {
    fn render(
        &self,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
        state: &mut AppState<C>,
    ) {
        let items = match state.evo.get_species() {
            Some(species) => species,
            None => return,
        };

        let inner_width = area.width.saturating_sub(2) as usize;
        let n = items.len();

        let bars = if inner_width == 0 || n == 0 {
            Vec::new()
        } else {
            let base = inner_width / n;
            let extra = inner_width % n;
            items
                .iter()
                .enumerate()
                .flat_map(|(i, species)| {
                    let color = match state.tables.species.selected_value {
                        Some(selected_id) if selected_id == species.id => {
                            crate::styles::SELECTED_GREEN
                        }
                        _ => Color::DarkGray,
                    };
                    let width = base + if i < extra { 1 } else { 0 };
                    (0..width).map(move |_| {
                        SparklineBar::from(species.size as u64).style(Style::default().fg(color))
                    })
                })
                .collect::<Vec<_>>()
        };

        let sparkline = Sparkline::default()
            .block(
                crate::styles::panel_block(state.nav.is_pane_focused(Pane::Chart)).title(
                    Line::from(Span::from(" Species Sizes ").fg(Color::White).bold()).centered(),
                ),
            )
            .data(bars)
            .direction(RenderDirection::LeftToRight)
            .style(Style::default().fg(Color::LightGreen))
            .max(items.iter().map(|s| s.size).max().unwrap_or(1) as u64);

        sparkline.render(area, buf);
    }
}

// // --- Generic, data-driven compact charts ---
// //
// // Mirror `LineChartWidget`: plain ratatui `Widget`s over a series of bar heights
// // (e.g. histogram bin counts), with a builder + `From` conveniences. Single
// // color — for per-bar coloring drive the underlying widget directly, as
// // `SpeciesSparklineComponent` does.

// /// A sparkline over a series of values — dense, sub-character bar heights, no
// /// axis chrome. Good for many-bin histograms in a narrow panel.
// pub struct SparklineWidget<'a> {
//     data: Vec<u64>,
//     color: Color,
//     max: Option<u64>,
//     direction: RenderDirection,
//     block: Option<Block<'a>>,
// }

// impl<'a> SparklineWidget<'a> {
//     pub fn new(data: Vec<u64>) -> Self {
//         Self {
//             data,
//             color: Color::LightCyan,
//             max: None,
//             direction: RenderDirection::LeftToRight,
//             block: None,
//         }
//     }

//     pub fn with_color(mut self, color: Color) -> Self {
//         self.color = color;
//         self
//     }

//     /// Fix the height scale (defaults to the series max), so several sparklines
//     /// can share a common scale.
//     pub fn with_max(mut self, max: u64) -> Self {
//         self.max = Some(max);
//         self
//     }

//     pub fn with_direction(mut self, direction: RenderDirection) -> Self {
//         self.direction = direction;
//         self
//     }

//     pub fn with_block(mut self, block: Block<'a>) -> Self {
//         self.block = Some(block);
//         self
//     }

//     pub fn titled(self, title: impl Into<String>) -> Self {
//         let block = Block::bordered().title(Line::from(format!(" {} ", title.into())).centered());
//         self.with_block(block)
//     }
// }

// impl From<Vec<u64>> for SparklineWidget<'_> {
//     fn from(data: Vec<u64>) -> Self {
//         Self::new(data)
//     }
// }

// impl From<&[u64]> for SparklineWidget<'_> {
//     fn from(data: &[u64]) -> Self {
//         Self::new(data.to_vec())
//     }
// }

// impl Widget for SparklineWidget<'_> {
//     fn render(self, area: Rect, buf: &mut Buffer) {
//         let max = self
//             .max
//             .unwrap_or_else(|| self.data.iter().copied().max().unwrap_or(1));
//         let bars = self
//             .data
//             .iter()
//             .map(|&v| SparklineBar::from(v))
//             .collect::<Vec<_>>();

//         let mut sparkline = Sparkline::default()
//             .data(bars)
//             .direction(self.direction)
//             .style(Style::default().fg(self.color))
//             .max(max);

//         if let Some(block) = self.block {
//             sparkline = sparkline.block(block);
//         }

//         sparkline.render(area, buf);
//     }
// }

// /// A bar chart over a series of values — wider, optionally value-labeled bars.
// /// Prefer [`SparklineWidget`] for dense histograms; use this when you want a
// /// handful of legible, labeled buckets.
// pub struct BarChartWidget<'a> {
//     data: Vec<u64>,
//     color: Color,
//     max: Option<u64>,
//     bar_width: u16,
//     bar_gap: u16,
//     show_values: bool,
//     block: Option<Block<'a>>,
// }

// impl<'a> BarChartWidget<'a> {
//     pub fn new(data: Vec<u64>) -> Self {
//         Self {
//             data,
//             color: Color::LightCyan,
//             max: None,
//             bar_width: 1,
//             bar_gap: 0,
//             show_values: false,
//             block: None,
//         }
//     }

//     pub fn with_color(mut self, color: Color) -> Self {
//         self.color = color;
//         self
//     }

//     pub fn with_max(mut self, max: u64) -> Self {
//         self.max = Some(max);
//         self
//     }

//     pub fn with_bar_width(mut self, width: u16) -> Self {
//         self.bar_width = width.max(1);
//         self
//     }

//     pub fn with_bar_gap(mut self, gap: u16) -> Self {
//         self.bar_gap = gap;
//         self
//     }

//     /// Print each bar's value inside it.
//     pub fn with_values(mut self, show: bool) -> Self {
//         self.show_values = show;
//         self
//     }

//     pub fn with_block(mut self, block: Block<'a>) -> Self {
//         self.block = Some(block);
//         self
//     }

//     pub fn titled(self, title: impl Into<String>) -> Self {
//         let block = Block::bordered().title(Line::from(format!(" {} ", title.into())).centered());
//         self.with_block(block)
//     }
// }

// impl From<Vec<u64>> for BarChartWidget<'_> {
//     fn from(data: Vec<u64>) -> Self {
//         Self::new(data)
//     }
// }

// impl From<&[u64]> for BarChartWidget<'_> {
//     fn from(data: &[u64]) -> Self {
//         Self::new(data.to_vec())
//     }
// }

// impl Widget for BarChartWidget<'_> {
//     fn render(self, area: Rect, buf: &mut Buffer) {
//         let bars = self
//             .data
//             .iter()
//             .map(|&v| {
//                 let text = if self.show_values {
//                     v.to_string()
//                 } else {
//                     String::new()
//                 };
//                 Bar::default()
//                     .value(v)
//                     .text_value(text)
//                     .style(Style::default().fg(self.color))
//             })
//             .collect::<Vec<_>>();

//         let mut chart = BarChart::default()
//             .data(BarGroup::default().bars(&bars))
//             .bar_width(self.bar_width)
//             .bar_gap(self.bar_gap);

//         if let Some(max) = self.max {
//             chart = chart.max(max);
//         }
//         if let Some(block) = self.block {
//             chart = chart.block(block);
//         }

//         chart.render(area, buf);
//     }
// }
