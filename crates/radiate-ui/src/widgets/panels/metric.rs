use crate::chart::DistributionLineChart;
use crate::state::{AppState, MetricChartType, Pane};
use crate::styles;
use crate::widgets::{AppWidget, FnWidget, LineChartWidget, Panel};
use radiate_engines::stats::{TagType, fmt_duration};
use radiate_engines::{Chromosome, Metric};
use radiate_utils::SmallStr;
use ratatui::prelude::*;
use ratatui::style::{Color, Stylize};
use ratatui::text::ToSpan;
use ratatui::widgets::canvas::Canvas;
use ratatui::widgets::{Paragraph, Row, Table, canvas::Line as CanvasLine};

#[derive(Default)]
pub struct MetricLineChartWidget {
    name: Option<SmallStr>,
    chart_type: Option<MetricChartType>,
    show_bottom_options: bool,
    show_x_axis: bool,
}

impl MetricLineChartWidget {
    pub fn new(name: impl Into<SmallStr>, chart_type: MetricChartType) -> Self {
        Self {
            name: Some(name.into()),
            chart_type: Some(chart_type),
            show_bottom_options: false,
            show_x_axis: false,
        }
    }

    pub fn with_show_bottom_options(mut self, show: bool) -> Self {
        self.show_bottom_options = show;
        self
    }

    pub fn with_show_x_axis(mut self, show: bool) -> Self {
        self.show_x_axis = show;
        self
    }
}

impl<C: Chromosome> AppWidget<C> for MetricLineChartWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut AppState<C>) {
        let current_metric_name = self
            .name
            .as_deref()
            .unwrap_or_else(|| state.get_selected_metric().unwrap_or(""));
        let Some(metric) = state.evo.metrics.get(current_metric_name) else {
            Paragraph::new(Line::from("No metric selected").centered()).render(area, buf);
            return;
        };

        let inner = if area.width > 2 && area.height > 2 {
            Rect {
                x: area.x + 1,
                y: area.y + 1,
                width: area.width.saturating_sub(2),
                height: area.height.saturating_sub(2),
            }
        } else {
            area
        };

        let chart_type = self
            .chart_type
            .unwrap_or_else(|| state.current_chart_view());

        let show_x = self.show_x_axis;

        if metric.tags().has(TagType::Statistic) {
            render_stat_metric_chart(chart_type, metric, show_x, inner, buf, state);
        } else if metric.tags().has(TagType::Distribution) {
            if chart_type == MetricChartType::BoxWhisker {
                let chart_metrics = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(8), Constraint::Length(1)].as_ref())
                    .split(inner);
                render_box_whisker_chart(metric, &chart_metrics, buf);
            } else if chart_type == MetricChartType::Distribution {
                render_distribution_metric_chart(metric, show_x, inner, buf);
            } else {
                render_stat_metric_chart(chart_type, metric, show_x, inner, buf, state);
            }
        } else {
            render_stat_metric_chart(chart_type, metric, show_x, inner, buf, state);
        }

        crate::styles::panel_block(state.nav.is_pane_focused(Pane::Chart) && self.name.is_none())
            .title_bottom(if self.show_bottom_options {
                chart_type_bottom(chart_type, state).centered()
            } else {
                Line::default()
            })
            .title_top(if self.show_bottom_options {
                Line::from(Span::styled(
                    format!(" [Tab] "),
                    Style::default().fg(Color::Green).bold(),
                ))
                .right_aligned()
            } else {
                Line::default()
            })
            .title(Line::from(format!(" {} ", current_metric_name,)).centered())
            .render(area, buf);
    }
}

pub struct MetricDetailPanelWidget;

impl<C: Chromosome> AppWidget<C> for MetricDetailPanelWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut AppState<C>) {
        let current_metric_name = state.get_selected_metric().unwrap_or("");
        let metrics = &state.evo.metrics;
        let metric = metrics.get(current_metric_name);

        let Some(metric) = metric else {
            Paragraph::new(Line::from("No metric selected").centered()).render(area, buf);
            return;
        };

        let rows = if metric.tags().has(TagType::Statistic) {
            map_to_stat_metric_rows(metric)
        } else if metric.tags().has(TagType::Time) {
            map_to_time_metric_rows(metric)
        } else if metric.tags().has(TagType::Distribution) {
            map_to_distribution_metric_rows(metric)
        } else {
            vec![
                Row::new(vec!["Key".bold(), current_metric_name.into()]),
                Row::new(vec!["Type".bold(), "Unknown".into()]),
            ]
        };

        let metric_tags = metric
            .tags()
            .iter()
            .map(|t| Row::new(vec![format!("{t:?}").bold()]))
            .collect::<Vec<_>>();

        let tag_table = Table::default()
            .rows(crate::styles::striped_rows(metric_tags))
            .widths([Constraint::Fill(1)]);

        let metric_table = Table::default()
            .rows(crate::styles::striped_rows(rows))
            .style(Style::default().fg(Color::White))
            .widths([Constraint::Fill(1), Constraint::Fill(1)]);

        let left_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);

        Panel::new(FnWidget::new(|area, buf| {
            Widget::render(metric_table, area, buf);
        }))
        .titled(format!(" {} ", current_metric_name).bold())
        .render(left_layout[0], buf);

        Panel::new(FnWidget::new(|area, buf| {
            Widget::render(tag_table, area, buf);
        }))
        .titled("Tags".to_span().bold())
        .render(left_layout[1], buf);
    }
}

fn render_stat_metric_chart(
    chart_type: MetricChartType,
    current_metric: &Metric,
    show_x_axis: bool,
    area: Rect,
    buf: &mut Buffer,
    state: &AppState<impl Chromosome>,
) {
    let charts = state
        .evo
        .get_chart_by_key(current_metric.name(), chart_type);

    LineChartWidget::from(charts)
        .with_show_x_axis(show_x_axis)
        .with_show_boarders(false)
        .render(area, buf);
}

fn render_box_whisker_chart(current_metric: &Metric, areas: &[Rect], buf: &mut Buffer) {
    if let Some(view) = current_metric.distributions() {
        let quantiles = view.quantiles(&[0.25, 0.5, 0.75]).unwrap_or(vec![0.0; 3]);
        let mean = view.mean();
        let min = view.min();
        let max = view.max();
        let stddev = view.stddev();
        let count = view.count();
        let q1 = quantiles[0];
        let med = quantiles[1];
        let q3 = quantiles[2];

        let canvas = Canvas::default()
            .x_bounds([(min - stddev) as f64, (max + stddev) as f64])
            .y_bounds([-1.0, 1.0])
            .background_color(styles::ALT_BG_COLOR)
            .paint(move |ctx| {
                if count >= 2 {
                    // Box (Q1 to Q3)
                    draw_line(ctx, q1, -0.4, q3, -0.4, Color::White);
                    draw_line(ctx, q1, 0.4, q3, 0.4, Color::White);
                    draw_line(ctx, q1, -0.4, q1, 0.4, Color::White);
                    draw_line(ctx, q3, -0.4, q3, 0.4, Color::White);

                    // Median
                    draw_line(ctx, med, -0.4, med, 0.4, Color::Yellow);

                    // Mean
                    draw_line(ctx, mean, -0.4, mean, 0.4, Color::Cyan);

                    // Whiskers
                    draw_line(ctx, min, 0.0, q1, 0.0, Color::White);
                    draw_line(ctx, q3, 0.0, max, 0.0, Color::White);

                    // Whisker caps
                    draw_line(ctx, min, -0.2, min, 0.2, Color::White);
                    draw_line(ctx, max, -0.2, max, 0.2, Color::White);
                } else {
                    // Single sample: just mark the point
                    draw_line(ctx, med, -0.4, med, 0.4, Color::Yellow);
                }
            });

        canvas.render(areas[0], buf);

        Paragraph::new(box_summary_line(q1, med, q3, mean)).render(areas[1], buf);
    } else {
        Paragraph::new(Line::from("No distribution data").centered())
            .style(Style::default().fg(Color::DarkGray).italic())
            .render(areas[0], buf);
    }
}

fn render_distribution_metric_chart(
    metric: &Metric,
    show_x_axis: bool,
    area: Rect,
    buf: &mut Buffer,
) {
    if let Some(view) = metric.distributions() {
        let chart = view
            .samples()
            .map(|samples| DistributionLineChart::from(samples).with_color(Color::LightGreen));

        LineChartWidget::from(chart.as_ref())
            .with_show_x_axis(show_x_axis)
            .with_show_boarders(false)
            .render(area, buf);
    }
}

/// Helper function to draw a line on a canvas
pub fn draw_line(
    ctx: &mut ratatui::widgets::canvas::Context,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    color: Color,
) {
    ctx.draw(&CanvasLine {
        x1: x1 as f64,
        y1: y1 as f64,
        x2: x2 as f64,
        y2: y2 as f64,
        color,
    });
}

fn chart_type_bottom(
    chart_type: MetricChartType,
    state: &AppState<impl Chromosome>,
) -> Line<'static> {
    let bottom = state
        .selected_metric_views()
        .iter()
        .map(|v| {
            if *v == chart_type {
                Span::styled(
                    format!(" {} ", v.short_label()),
                    Style::default().fg(Color::LightGreen),
                )
            } else {
                Span::styled(
                    format!(" {} ", v.short_label()),
                    Style::default().fg(Color::White),
                )
            }
        })
        .collect::<Vec<_>>();

    let mut final_bottom = Vec::new();
    for i in 0..bottom.len() {
        final_bottom.push(bottom[i].clone());
        if i == bottom.len() - 1 {
            break;
        }

        final_bottom.push(Span::styled("|", Style::default().fg(Color::White)));
    }

    Line::from(final_bottom)
}

fn box_summary_line<'a>(q1: f32, med: f32, q3: f32, mean: f32) -> Line<'a> {
    let kv = |label: &str, val: f32, color: Color| {
        [
            Span::styled(format!("{label} "), Style::default().fg(Color::Gray)),
            Span::styled(format!("{val:.2}"), Style::default().fg(color)),
            Span::raw("  "),
        ]
    };
    Line::from(
        [
            kv("q1", q1, Color::White),
            kv("med", med, Color::Yellow), // ← matches the median line
            kv("μ", mean, Color::Cyan),    // ← matches the mean line
            kv("q3", q3, Color::White),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>(),
    )
    .centered()
}

fn map_to_stat_metric_rows(metric: &Metric) -> Vec<Row<'_>> {
    if let Some(view) = metric.stats() {
        let rows = vec![
            Row::new(vec!["Type".bold(), metric.dtype().to_string().into()]),
            Row::new(vec!["Gen.".bold(), metric.generation().to_string().into()]),
            Row::new(vec![
                "Updates".bold(),
                metric.update_count().to_string().into(),
            ]),
            Row::new(vec!["Last".bold(), format!("{:.2}", view.last()).into()]),
            Row::new(vec!["Sum".bold(), format!("{:.4}", view.sum()).into()]),
            Row::new(vec!["Min.".bold(), format!("{:.2}", view.min()).into()]),
            Row::new(vec!["Max.".bold(), format!("{:.2}", view.max()).into()]),
            Row::new(vec!["Mean".bold(), format!("{:.4}", view.mean()).into()]),
            Row::new(vec![
                "Std Dev".bold(),
                format!("{:.4}", view.stddev()).into(),
            ]),
            Row::new(vec!["Var.".bold(), format!("{:.4}", view.var()).into()]),
            Row::new(vec![
                "Skew".bold(),
                format!("{:.4}", view.skewness()).into(),
            ]),
            Row::new(vec![
                "Kurt.".bold(),
                format!("{:.4}", view.kurtosis()).into(),
            ]),
        ];

        return rows;
    }

    vec![]
}

fn map_to_time_metric_rows(metric: &Metric) -> Vec<Row<'_>> {
    if let Some(view) = metric.times() {
        let rows = vec![
            Row::new(vec!["Type".bold(), metric.dtype().to_string().into()]),
            Row::new(vec!["Gen.".bold(), metric.generation().to_string().into()]),
            Row::new(vec![
                "Updates".bold(),
                metric.update_count().to_string().into(),
            ]),
            Row::new(vec!["Last".bold(), fmt_duration(view.last()).into()]),
            Row::new(vec!["Sum".bold(), fmt_duration(view.sum()).into()]),
            Row::new(vec!["Min.".bold(), fmt_duration(view.min()).into()]),
            Row::new(vec!["Max.".bold(), fmt_duration(view.max()).into()]),
            Row::new(vec!["Mean".bold(), fmt_duration(view.mean()).into()]),
            Row::new(vec!["Std Dev".bold(), fmt_duration(view.stddev()).into()]),
            Row::new(vec!["Var.".bold(), fmt_duration(view.var()).into()]),
            Row::new(vec!["Skew".bold(), fmt_duration(view.skewness()).into()]),
            Row::new(vec!["Kurt.".bold(), fmt_duration(view.kurtosis()).into()]),
        ];

        return rows;
    }

    vec![]
}

fn map_to_distribution_metric_rows(metric: &Metric) -> Vec<Row<'_>> {
    if let Some(view) = metric.distributions() {
        let rows = vec![
            Row::new(vec!["Type".bold(), metric.dtype().to_string().into()]),
            Row::new(vec!["Gen.".bold(), metric.generation().to_string().into()]),
            Row::new(vec![
                "Updates".bold(),
                metric.update_count().to_string().into(),
            ]),
            Row::new(vec!["Count".bold(), view.count().to_string().into()]),
            Row::new(vec!["Sum".bold(), format!("{:.4}", view.sum()).into()]),
            Row::new(vec!["Min.".bold(), format!("{:.2}", view.min()).into()]),
            Row::new(vec!["Max.".bold(), format!("{:.2}", view.max()).into()]),
            Row::new(vec!["Mean".bold(), format!("{:.4}", view.mean()).into()]),
            Row::new(vec![
                "Std Dev".bold(),
                format!("{:.4}", view.stddev()).into(),
            ]),
            Row::new(vec!["Var.".bold(), format!("{:.4}", view.var()).into()]),
            Row::new(vec![
                "Skew".bold(),
                format!("{:.4}", view.skewness()).into(),
            ]),
            Row::new(vec![
                "Kurt.".bold(),
                format!("{:.4}", view.kurtosis()).into(),
            ]),
            Row::new(vec![
                "q50".bold(),
                format!("{:.4}", view.quantile(0.5).unwrap_or(f32::NAN)).into(),
            ]),
        ];

        return rows;
    }

    vec![]
}

// /// Bin `samples` into `bins` equal-width buckets over [min,max] → per-bucket counts.
// /// Degenerate input (empty / all-equal) collapses into one bucket.
// fn histogram(samples: &[f32], bins: usize) -> Vec<u64> {
//     let n = bins.max(1);
//     let mut counts = vec![0u64; n];

//     let (min, max) = samples
//         .iter()
//         .copied()
//         .filter(|v| v.is_finite())
//         .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), v| {
//             (lo.min(v), hi.max(v))
//         });

//     if !min.is_finite() || max <= min {
//         counts[0] = samples.iter().filter(|v| v.is_finite()).count() as u64;
//         return counts;
//     }

//     let span = max - min;
//     for &v in samples.iter().filter(|v| v.is_finite()) {
//         let idx = (((v - min) / span) * n as f32) as usize;
//         counts[idx.min(n - 1)] += 1; // clamp the max edge into the last bin
//     }
//     counts
// }

// pub struct MetricBoxWhiskerChartWidget;

// impl<C: Chromosome> AppWidget<C> for MetricBoxWhiskerChartWidget {
//     fn render(&self, area: Rect, buf: &mut Buffer, state: &mut AppState<C>) {
//         let current_metric_name = state.get_selected_metric().unwrap_or("");
//         let Some(current_metric) = state.evo.metrics.get(current_metric_name) else {
//             Paragraph::new(Line::from("No metric selected").centered()).render(area, buf);
//             return;
//         };

//         let inner = if area.width > 2 && area.height > 2 {
//             Rect {
//                 x: area.x + 1,
//                 y: area.y + 1,
//                 width: area.width.saturating_sub(2),
//                 height: area.height.saturating_sub(2),
//             }
//         } else {
//             area
//         };

//         let chart_metrics = Layout::default()
//             .direction(Direction::Vertical)
//             .constraints([Constraint::Min(8), Constraint::Length(1)].as_ref())
//             .split(inner);

//         if let Some(view) = current_metric.distributions() {
//             let quantiles = view.quantiles(&[0.25, 0.5, 0.75]).unwrap_or(vec![0.0; 3]);
//             let mean = view.mean();
//             let min = view.min();
//             let max = view.max();
//             let stddev = view.stddev();
//             let count = view.count();
//             let q1 = quantiles[0];
//             let med = quantiles[1];
//             let q3 = quantiles[2];

//             let canvas = Canvas::default()
//                 .x_bounds([(min - stddev) as f64, (max + stddev) as f64])
//                 .y_bounds([-1.0, 1.0])
//                 .background_color(styles::ALT_BG_COLOR)
//                 .paint(move |ctx| {
//                     if count >= 2 {
//                         // Box (Q1 to Q3)
//                         draw_line(ctx, q1, -0.4, q3, -0.4, Color::White);
//                         draw_line(ctx, q1, 0.4, q3, 0.4, Color::White);
//                         draw_line(ctx, q1, -0.4, q1, 0.4, Color::White);
//                         draw_line(ctx, q3, -0.4, q3, 0.4, Color::White);

//                         // Median
//                         draw_line(ctx, med, -0.4, med, 0.4, Color::Yellow);

//                         // Mean
//                         draw_line(ctx, mean, -0.4, mean, 0.4, Color::Cyan);

//                         // Whiskers
//                         draw_line(ctx, min, 0.0, q1, 0.0, Color::White);
//                         draw_line(ctx, q3, 0.0, max, 0.0, Color::White);

//                         // Whisker caps
//                         draw_line(ctx, min, -0.2, min, 0.2, Color::White);
//                         draw_line(ctx, max, -0.2, max, 0.2, Color::White);
//                     } else {
//                         // Single sample: just mark the point
//                         draw_line(ctx, med, -0.4, med, 0.4, Color::Yellow);
//                     }
//                 });

//             canvas.render(chart_metrics[0], buf);

//             Paragraph::new(box_summary_line(q1, med, q3, mean)).render(chart_metrics[1], buf);
//         } else {
//             Paragraph::new(Line::from("No distribution data").centered())
//                 .style(Style::default().fg(Color::DarkGray).italic())
//                 .render(chart_metrics[0], buf);
//         }

//         crate::styles::panel_block(state.nav.is_pane_focused(Pane::Chart))
//             .title(
//                 Line::from(format!(" {} ", current_metric_name))
//                     .fg(crate::styles::SELECTED_GREEN)
//                     .centered(),
//             )
//             .render(area, buf);
//     }
// }
