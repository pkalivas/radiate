use crate::{state::AppState, styles, widgets::Panel};
use radiate_engines::Chromosome;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols,
    text::Line,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Widget},
};

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

pub struct ParetoPagingWidget<'a, C>
where
    C: Chromosome,
{
    state: &'a AppState<C>,
}

impl<'a, C> ParetoPagingWidget<'a, C>
where
    C: Chromosome,
{
    pub fn new(state: &'a AppState<C>) -> Self {
        Self { state }
    }
}

impl<'a, C> Widget for ParetoPagingWidget<'a, C>
where
    C: Chromosome,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let d = self.state.objective_state.objective.dims();
        if d < 2 {
            Panel::empty("need 2+ objectives").render(area, buf);
            return;
        }

        let total = num_pairs(d);
        if total == 0 {
            Panel::empty("no objective pairs")
                .titled(Line::from(" Pareto Front ").centered())
                .render(area, buf);
            return;
        }

        let objective_state = &self.state.objective_state;
        let start = objective_state
            .chart_start_index
            .min(total.saturating_sub(1));
        let count = objective_state.charts_visible.max(1).min(total);

        let title = format!(" Pareto Front ({}/{} pairs of obj{}D) ", count, total, d);

        Panel::new(ParetoPagerInner {
            state: self.state,
            start,
            count,
            d,
            total,
        })
        .titled(Line::from(title).centered())
        .render(area, buf);
    }
}

struct ParetoPagerInner<'a, C>
where
    C: Chromosome,
{
    state: &'a AppState<C>,
    start: usize,
    count: usize,
    d: usize,
    total: usize,
}

impl<'a, C> Widget for ParetoPagerInner<'a, C>
where
    C: Chromosome,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let areas =
            Layout::horizontal(std::iter::repeat(Constraint::Fill(1)).take(self.count)).split(area);

        for (pane_idx, rect) in areas.iter().enumerate() {
            let k = self.start + pane_idx;
            if k >= self.total {
                break;
            }
            let (i, j) = kth_pair(k, self.d);
            ParetoPlot::new(self.state, i, j).render(*rect, buf);
        }
    }
}

pub struct ParetoPlot<'a, C>
where
    C: Chromosome,
{
    state: &'a AppState<C>,
    i: usize,
    j: usize,
}

impl<'a, C> ParetoPlot<'a, C>
where
    C: Chromosome,
{
    pub fn new(state: &'a AppState<C>, i: usize, j: usize) -> Self {
        Self { state, i, j }
    }
}

impl<'a, C> Widget for ParetoPlot<'a, C>
where
    C: Chromosome,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let front = match &self.state.front {
            Some(f) if !f.is_empty() => f,
            _ => {
                Block::default()
                    .borders(Borders::ALL)
                    .title(Line::from(" Pareto Front (no data) ").centered())
                    .render(area, buf);
                return;
            }
        };

        let mut points: Vec<(f64, f64)> = Vec::new();
        let (mut min_x, mut max_x) = (f64::INFINITY, f64::NEG_INFINITY);
        let (mut min_y, mut max_y) = (f64::INFINITY, f64::NEG_INFINITY);

        for p in front.values().iter() {
            let score = match p.score() {
                Some(s) => s,
                None => continue,
            };
            let s = score.as_ref();

            if self.i >= s.len() || self.j >= s.len() {
                continue;
            }

            let x = s[self.i] as f64;
            let y = s[self.j] as f64;

            points.push((x, y));
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }

        if points.is_empty() {
            Block::default()
                .borders(Borders::ALL)
                .title(Line::from(" Pareto Front (no points) ").centered())
                .render(area, buf);
            return;
        }

        let trim = 0.02; // make this a UI setting if you want
        let (points, bounds) = filter_outliers_quantile(&points, trim);

        let (mut min_x, mut max_x, mut min_y, mut max_y) =
            (bounds[0], bounds[1], bounds[2], bounds[3]);

        // Avoid zero range
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
            .bg(styles::ALT_BG_COLOR)
            .block(
                Block::default()
                    .title_top(Line::from(format!(" obj{} vs obj{} ", self.i, self.j)).centered()),
            )
            .x_axis(
                Axis::default()
                    .title(format!("D({})", self.i).bg(styles::ALT_BG_COLOR))
                    .style(Style::default().gray())
                    .bounds([min_x, max_x]),
            )
            .y_axis(
                Axis::default()
                    .title(format!("D({})", self.j).bg(styles::ALT_BG_COLOR))
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

fn quantile(sorted: &[f64], q: f64) -> f64 {
    let n = sorted.len() as f64;
    let pos = (n - 1.0) * q.clamp(0.0, 1.0);
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;

    if lo == hi {
        return sorted[lo];
    }

    let t = pos - (lo as f64);
    sorted[lo] * (1.0 - t) + sorted[hi] * t
}

fn filter_outliers_quantile(points: &[(f64, f64)], trim: f64) -> (Vec<(f64, f64)>, [f64; 4]) {
    // trim=0.02 => keep [2%, 98%]
    let lo_q = trim;
    let hi_q = 1.0 - trim;

    let mut xs = points.iter().map(|(x, _)| *x).collect::<Vec<f64>>();
    let mut ys = points.iter().map(|(_, y)| *y).collect::<Vec<f64>>();
    xs.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    ys.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

    let x_lo = quantile(&xs, lo_q);
    let x_hi = quantile(&xs, hi_q);
    let y_lo = quantile(&ys, lo_q);
    let y_hi = quantile(&ys, hi_q);

    let filtered: Vec<(f64, f64)> = points
        .iter()
        .copied()
        .filter(|(x, y)| *x >= x_lo && *x <= x_hi && *y >= y_lo && *y <= y_hi)
        .collect();

    (filtered, [x_lo, x_hi, y_lo, y_hi])
}
