use crate::state::AppState;
use crate::styles::{self, striped_rows};
use crate::widgets::AppWidget;
use radiate_engines::{Chromosome, stats::fmt_duration};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Cell, Row, Table, Widget};
use ratatui::{buffer::Buffer, text::Span};
use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::Stylize,
};
use std::fmt::Debug;
use std::time::Duration;

/// Column titles, and how many leading columns are text. The rest are
/// numbers, and their titles are right-aligned to sit over them.
const HANDLER_HEADER: ([&str; 7], usize) = (
    [
        "Handler", "Event", "Handled", "Max/s", "Time", "Wait", "Queued",
    ],
    2,
);
const EVENT_HEADER: ([&str; 6], usize) = (
    [
        "Event",
        "Published",
        "Rate",
        "Cost",
        "Unbuilt",
        "Undelivered",
    ],
    1,
);

/// A queue this deep is shown as a problem whether or not it's still growing.
const QUEUED_ALERT: u64 = 1_000;

/// Every live subscription on the engine's event stream: how much it has
/// handled, how fast it could go, how long its handler takes and its events
/// wait (over the last window), and how far behind it is. Its title carries
/// the whole stream's throughput and utilization.
pub struct StreamHandlersWidget;

/// Every event type on the engine's event stream: how often it's published,
/// how much handler time each publish costs, and how often laziness or
/// schedules meant nobody got it.
pub struct StreamEventsWidget;

impl<C: Chromosome + Debug> AppWidget<C> for StreamHandlersWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut AppState<C>) {
        let mut previous = "";
        let rows = state
            .stream
            .handlers
            .iter()
            .map(|row| {
                // Show each handler's name once, on the first row of its group.
                let handler = if row.handler == previous {
                    ""
                } else {
                    row.handler.as_str()
                };
                previous = &row.handler;

                Row::new(vec![
                    text_cell(handler),
                    text_cell(&row.event),
                    number_cell(row.handled),
                    rate_cell(row.capacity),
                    duration_cell(row.time),
                    duration_cell(row.wait),
                    queued_cell(row.queued, row.growing),
                ])
            })
            .collect::<Vec<_>>();

        let stream = &state.stream;
        let summary = match (stream.throughput, stream.utilization) {
            (Some(throughput), Some(utilization)) => Some(format!(
                " {} · {:.1}% busy ",
                fmt_rate(throughput),
                utilization * 100.0
            )),
            _ => None,
        };

        render_table(
            area,
            buf,
            " Handlers ",
            summary,
            HANDLER_HEADER,
            rows,
            [
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Length(10),
                Constraint::Length(9),
                Constraint::Length(9),
                Constraint::Length(9),
                Constraint::Length(8),
            ],
        );
    }
}

impl<C: Chromosome + Debug> AppWidget<C> for StreamEventsWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut AppState<C>) {
        let rows = state
            .stream
            .events
            .iter()
            .map(|row| {
                Row::new(vec![
                    text_cell(&row.event),
                    number_cell(row.published),
                    rate_cell(row.rate),
                    duration_cell(row.cost),
                    dim_zero_cell(row.unbuilt),
                    dim_zero_cell(row.undelivered),
                ])
            })
            .collect::<Vec<_>>();

        render_table(
            area,
            buf,
            " Events ",
            None,
            EVENT_HEADER,
            rows,
            [
                Constraint::Fill(1),
                Constraint::Length(10),
                Constraint::Length(9),
                Constraint::Length(9),
                Constraint::Length(9),
                Constraint::Length(12),
            ],
        );
    }
}

fn render_table<const N: usize>(
    area: Rect,
    buf: &mut Buffer,
    title: &'static str,
    summary: Option<String>,
    (header, text_columns): ([&str; N], usize),
    rows: Vec<Row<'_>>,
    widths: [Constraint; N],
) {
    // Display only: never focused, so never highlighted.
    let mut block = styles::panel_block(false)
        .title(Line::from(vec![Span::from(title).fg(Color::White).bold()]).centered());
    if let Some(summary) = summary {
        block = block.title(
            Line::from(summary)
                .fg(styles::TEXT_FG_COLOR)
                .right_aligned(),
        );
    }
    let table = Table::default()
        .block(block)
        .header(header_row(&header, text_columns))
        .rows(striped_rows(rows))
        .column_spacing(1)
        .style(Color::White)
        .widths(widths);

    Widget::render(table, area, buf);
}

fn header_row<'a>(titles: &[&'a str], text_columns: usize) -> Row<'a> {
    let cells = titles.iter().enumerate().map(|(i, &title)| {
        let alignment = if i < text_columns {
            Alignment::Left
        } else {
            Alignment::Center
        };
        Cell::from(Text::from(title).alignment(alignment))
    });

    Row::new(cells)
        .height(1)
        .style(Style::default().bold().underlined().fg(Color::White))
}

fn text_cell(text: &str) -> Cell<'_> {
    Cell::from(text).style(Style::default().bold())
}

fn center(text: String) -> Text<'static> {
    Text::from(text).alignment(Alignment::Center)
}

fn number_cell(n: u64) -> Cell<'static> {
    Cell::from(center(styles::format_thousands(n as usize))).style(Style::default().bold())
}

/// Zero is the normal case, so it fades out and anything else stands out.
fn dim_zero_cell(n: u64) -> Cell<'static> {
    let color = if n == 0 {
        styles::TREND_FLAT_COLOR
    } else {
        styles::TEXT_FG_COLOR
    };
    Cell::from(center(styles::format_thousands(n as usize))).style(Style::default().fg(color))
}

fn rate_cell(rate: Option<f64>) -> Cell<'static> {
    let text = rate.map_or_else(|| "-".to_string(), fmt_rate);
    Cell::from(center(text)).style(Style::default().fg(styles::TEXT_FG_COLOR))
}

fn fmt_rate(rate: f64) -> String {
    match rate {
        r if r >= 1_000_000.0 => format!("{:.1}M/s", r / 1_000_000.0),
        r if r >= 1_000.0 => format!("{:.1}k/s", r / 1_000.0),
        r if r >= 10.0 => format!("{r:.0}/s"),
        r => format!("{r:.1}/s"),
    }
}

/// A duration, or `-` before there's anything to measure.
fn duration_cell(duration: Option<Duration>) -> Cell<'static> {
    let text = duration.map_or_else(|| "-".to_string(), fmt_duration);
    Cell::from(center(text)).style(Style::default().fg(styles::TEXT_FG_COLOR))
}

/// Grey when empty, yellow while it's growing, red once it's deep.
fn queued_cell(queued: u64, growing: bool) -> Cell<'static> {
    let style = match queued {
        0 => Style::default().fg(styles::TREND_FLAT_COLOR),
        q if q >= QUEUED_ALERT => Style::default().fg(styles::TREND_DOWN_COLOR).bold(),
        _ if growing => Style::default().fg(styles::LOG_WARN_COLOR),
        _ => Style::default().fg(styles::TEXT_FG_COLOR),
    };
    Cell::from(center(styles::format_thousands(queued as usize))).style(style)
}
