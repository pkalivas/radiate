use crate::state::AppState;
use crate::widgets::{AppWidget, FnWidget, LineChartWidget, Panel};
use radiate_engines::stats::{TagType, fmt_duration};
use radiate_engines::{Chromosome, Metric};
use ratatui::prelude::*;
use ratatui::style::{Color, Stylize};
use ratatui::text::ToSpan;
use ratatui::widgets::{Block, Paragraph, Row, Table};

pub struct MetricChartPanelWidget;

impl<C: Chromosome> AppWidget<C> for MetricChartPanelWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut AppState<C>) {
        let current_metric_name = state.get_selected_metric().unwrap_or("");

        let chart_type = state.nav.chart_tab;
        let charts = state.evo.get_chart_by_key(current_metric_name, chart_type);

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

        let chart_metrics = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(8), Constraint::Length(1)].as_ref())
            .split(inner);

        LineChartWidget::from(charts)
            .with_show_x_axis(false)
            .with_show_boarders(false)
            .render(chart_metrics[0], buf);

        let line = if let Some(metric) = state.evo.metrics.get(current_metric_name) {
            render_metrics_text(metric)
        } else {
            Line::from("No data".to_span().italic().fg(Color::DarkGray))
        };

        Paragraph::new(line)
            .alignment(Alignment::Center)
            .render(chart_metrics[1], buf);

        Block::bordered()
            .title(Line::from(format!(" {} ", current_metric_name)).centered())
            .render(area, buf);
    }
}

fn render_metrics_text<'a>(metrics: &Metric) -> Line<'a> {
    let spans = if let Some(stat_view) = metrics.stats() {
        vec![
            Span::styled("last", Style::default().fg(Color::Gray)),
            Span::styled(
                format!(" {:.4}", stat_view.last()),
                Style::default().fg(Color::LightGreen),
            ),
            Span::raw(" "),
        ]
        //     Span::styled("avg", Style::default().fg(Color::Gray)),
        //     Span::styled(format!(" {:.0}", mean_val), Style::default().fg(c)),
        //     Span::raw(" "),
        //     Span::styled("med", Style::default().fg(Color::Gray)),
        //     Span::styled(format!(" {:.0}", median_val), Style::default().fg(c)),
    } else {
        vec![Span::styled(
            "No data",
            Style::default().fg(Color::DarkGray).italic(),
        )]
    };

    // let spans = vec![
    //     Span::styled("avg", Style::default().fg(Color::Gray)),
    //     Span::styled(format!(" {:.0}", mean_val), Style::default().fg(c)),
    //     Span::raw(" "),
    //     Span::styled("med", Style::default().fg(Color::Gray)),
    //     Span::styled(format!(" {:.0}", median_val), Style::default().fg(c)),
    //     // … p25, p75, optional jit/loss …
    // ];
    Line::from(spans)
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
            .header(Row::new(vec![
                "Tags".to_span().bold().fg(crate::styles::SELECTED_GREEN),
            ]))
            .rows(crate::styles::striped_rows(metric_tags))
            .widths([Constraint::Fill(1)]);

        let metric_table = Table::default()
            .rows(crate::styles::striped_rows(rows))
            .style(Style::default().fg(Color::White))
            .widths([Constraint::Fill(1), Constraint::Fill(1)]);

        Panel::new(FnWidget::new(|area, buf| {
            let left_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(area);

            Widget::render(metric_table, left_layout[0], buf);
            Widget::render(tag_table, left_layout[1], buf);
        }))
        .titled(
            format!(" {} ", current_metric_name)
                .fg(crate::styles::SELECTED_GREEN)
                .bold(),
        )
        .render(area, buf);
    }
}

fn map_to_stat_metric_rows(metric: &Metric) -> Vec<Row<'_>> {
    if let Some(view) = metric.stats() {
        let rows = vec![
            Row::new(vec!["Type".bold(), metric.dtype().to_string().into()]),
            Row::new(vec![
                "Generation".bold(),
                metric.generation().to_string().into(),
            ]),
            Row::new(vec![
                "Updates".bold(),
                metric.update_count().to_string().into(),
            ]),
            Row::new(vec![
                "Last Value".bold(),
                format!("{:.2}", view.last()).into(),
            ]),
            Row::new(vec!["Sum".bold(), format!("{:.4}", view.sum()).into()]),
            Row::new(vec!["Min.".bold(), format!("{:.2}", view.min()).into()]),
            Row::new(vec!["Max.".bold(), format!("{:.2}", view.max()).into()]),
            Row::new(vec!["Mean".bold(), format!("{:.4}", view.mean()).into()]),
            Row::new(vec![
                "Std Dev".bold(),
                format!("{:.4}", view.stddev()).into(),
            ]),
            Row::new(vec!["Variance".bold(), format!("{:.4}", view.var()).into()]),
            Row::new(vec![
                "Skew".bold(),
                format!("{:.4}", view.skewness()).into(),
            ]),
            Row::new(vec![
                "Kurtosis".bold(),
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
            Row::new(vec![
                "Generation".bold(),
                metric.generation().to_string().into(),
            ]),
            Row::new(vec![
                "Updates".bold(),
                metric.update_count().to_string().into(),
            ]),
            Row::new(vec!["Last Value".bold(), fmt_duration(view.last()).into()]),
            Row::new(vec!["Sum".bold(), fmt_duration(view.sum()).into()]),
            Row::new(vec!["Min.".bold(), fmt_duration(view.min()).into()]),
            Row::new(vec!["Max.".bold(), fmt_duration(view.max()).into()]),
            Row::new(vec!["Mean".bold(), fmt_duration(view.mean()).into()]),
            Row::new(vec!["Std Dev".bold(), fmt_duration(view.stddev()).into()]),
            Row::new(vec!["Variance".bold(), fmt_duration(view.var()).into()]),
            Row::new(vec!["Skew".bold(), fmt_duration(view.skewness()).into()]),
            Row::new(vec![
                "Kurtosis".bold(),
                fmt_duration(view.kurtosis()).into(),
            ]),
        ];

        return rows;
    }

    vec![]
}

fn map_to_distribution_metric_rows(metric: &Metric) -> Vec<Row<'_>> {
    if let Some(view) = metric.distributions() {
        let rows = vec![
            Row::new(vec!["Type".bold(), metric.dtype().to_string().into()]),
            Row::new(vec![
                "Generation".bold(),
                metric.generation().to_string().into(),
            ]),
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
            Row::new(vec!["Variance".bold(), format!("{:.4}", view.var()).into()]),
            Row::new(vec![
                "Skew".bold(),
                format!("{:.4}", view.skewness()).into(),
            ]),
            Row::new(vec![
                "Kurtosis".bold(),
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
