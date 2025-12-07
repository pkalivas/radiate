use crate::state::AppState;
use crate::styles;
use color_eyre::owo_colors::style;
use radiate_engines::Chromosome;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols;
use ratatui::text::Line;
use ratatui::widgets::{Axis, Block, Chart, Dataset, GraphType, Widget};

#[inline]
pub fn num_pairs(d: usize) -> usize {
    d.saturating_sub(1) * d / 2
}

// Map k ∈ [0, nC2) to (i, j) with i < j
#[inline]
pub fn kth_pair(mut k: usize, d: usize) -> (usize, usize) {
    let mut i = 0;
    while i < d && k >= d - i - 1 {
        k -= d - i - 1;
        i += 1;
    }
    (i, i + 1 + k)
}

pub struct ParetoFrontTemp<'a, C>
where
    C: Chromosome,
{
    state: &'a AppState<C>,
}

impl<'a, C> ParetoFrontTemp<'a, C>
where
    C: Chromosome,
{
    pub fn new(state: &'a AppState<C>) -> Self {
        Self { state }
    }
}

pub struct ParetoFrontWidget<'a, C>
where
    C: Chromosome,
{
    state: &'a AppState<C>,
    obj_one: usize,
    obj_two: usize,
}

impl<'a, C> Widget for ParetoFrontTemp<'a, C>
where
    C: Chromosome,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let objective_state = &self.state.objective_state;
        let num_showing = objective_state.charts_visible;
        let total_num = num_pairs(objective_state.objective.dimensions());

        let title = format!(
            " Pareto Front ({}/{} pairs of obj{}D) ",
            num_showing,
            total_num,
            objective_state.objective.dimensions()
        );

        let block = Block::bordered().title(Line::from(title).centered());

        let inner = block.inner(area);
        let d = self.state.objective_state.objective.dimensions();
        let total = num_pairs(d);

        if total == 0 {
            ParetoFrontWidget::new(&self.state, 0, 1).render(inner, buf);
        } else {
            let start = self.state.objective_state.chart_start_index.min(total - 1);
            let count = self.state.objective_state.charts_visible;

            let areas =
                Layout::horizontal(std::iter::repeat(Constraint::Fill(1)).take(count)).split(inner);

            for (pane_idx, area_rect) in areas.iter().enumerate() {
                let k = start + pane_idx;
                let (i, j) = kth_pair(k, d);

                ParetoFrontWidget::new(&self.state, i, j).render(*area_rect, buf);
            }
        }

        block.render(area, buf);
    }
}

impl<'a, C> ParetoFrontWidget<'a, C>
where
    C: Chromosome,
{
    pub fn new(state: &'a AppState<C>, obj_one: usize, obj_two: usize) -> Self {
        Self {
            state,
            obj_one,
            obj_two,
        }
    }
}

impl<'a, C> Widget for ParetoFrontWidget<'a, C>
where
    C: Chromosome,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let front = match &self.state.front {
            Some(f) if !f.is_empty() => f,
            _ => {
                Block::bordered()
                    .title(Line::from(" Pareto Front (no data) ").centered())
                    .render(area, buf);
                return;
            }
        };

        let mut points: Vec<(f64, f64)> = Vec::new();
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for p in front.values().iter() {
            if let Some(score) = p.score() {
                let s = score.as_ref();
                if s.len() > self.obj_one && s.len() > self.obj_two {
                    let x = s[self.obj_one] as f64;
                    let y = s[self.obj_two] as f64;
                    points.push((x, y));

                    if x < min_x {
                        min_x = x;
                    }
                    if x > max_x {
                        max_x = x;
                    }
                    if y < min_y {
                        min_y = y;
                    }
                    if y > max_y {
                        max_y = y;
                    }
                }
            }
        }

        if points.is_empty() {
            Block::bordered()
                .title(Line::from(" Pareto Front (no data) ").centered())
                .render(area, buf);
            return;
        }

        // Avoid zero-width axes
        if (max_x - min_x).abs() < f64::EPSILON {
            min_x -= 0.5;
            max_x += 0.5;
        }
        if (max_y - min_y).abs() < f64::EPSILON {
            min_y -= 0.5;
            max_y += 0.5;
        }

        let mid_y = (min_y + max_y) / 2.0;

        let dataset = Dataset::default()
            .graph_type(GraphType::Scatter)
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::LightCyan))
            .data(&points);

        let chart = Chart::new(vec![dataset])
            .bg(styles::ALT_ROW_BG_COLOR)
            .block(Block::default().title_top(
                Line::from(format!("obj{} vs obj{}", self.obj_one, self.obj_two)).centered(),
            ))
            .x_axis(
                Axis::default()
                    .title(format!("D({})", self.obj_one).bg(styles::ALT_ROW_BG_COLOR))
                    .style(Style::default().gray())
                    .bounds([min_x, max_x])
                    .labels(Line::from(vec![
                        format!("{:.2}", min_x).bold().into(),
                        format!("{:.2}", (min_x + max_x) / 2.0).into(),
                        format!("{:.2}", max_x).bold().into(),
                    ])),
            )
            .y_axis(
                Axis::default()
                    .title(format!("D({})", self.obj_two).bg(styles::ALT_ROW_BG_COLOR))
                    .style(Style::default().gray())
                    .bounds([min_y, max_y])
                    .labels(Line::from(vec![
                        format!("{:.2}", min_y).bold().into(),
                        format!("{:.2}", mid_y).into(),
                        format!("{:.2}", max_y).bold().into(),
                    ])),
            );

        chart.render(area, buf);
    }
}
