use crate::state::{AppState, MetricsTab};
use crate::styles;
use crate::widgets::{ChartWidget, ParetoPagingWidget, StatsTableWidget, TimeTableWidget};
use crate::widgets::{EngineBaseWidget, FilterWidget};
use radiate_engines::Chromosome;
use ratatui::style::Stylize;
use ratatui::widgets::BorderType;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Tabs, Widget, Wrap},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PanelId {
    Engine,
    Fitness,
    Pareto,
    Filters,
    Metrics,
    Help,
}

pub trait UiPanel<C: Chromosome>: Send + Sync {
    fn id(&self) -> PanelId;
    fn block(&self, state: &AppState<C>) -> Option<Block<'static>> {
        if let Some(title) = self.title(state) {
            Some(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(title),
            )
        } else {
            None
        }
    }

    fn title(&self, _state: &AppState<C>) -> Option<Line<'static>> {
        None
    }
    fn render(&self, state: &mut AppState<C>, area: Rect, buf: &mut Buffer);
}

pub struct Panel<W> {
    title: Option<Line<'static>>,
    child: W,
}

impl<W> Panel<W>
where
    W: Widget,
{
    pub fn new(title: impl Into<Line<'static>>, child: W) -> Self {
        Self {
            title: Some(title.into()),
            child,
        }
    }

    pub fn untitled(child: W) -> Self {
        Self { title: None, child }
    }
}

impl<W> Widget for Panel<W>
where
    W: Widget,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.title.is_none() {
            self.child.render(area, buf);
            return;
        } else if let Some(title) = self.title {
            let block = Block::default().borders(Borders::ALL).title(title);
            let inner = block.inner(area);
            block.render(area, buf);
            self.child.render(inner, buf);
        }
    }
}

pub struct Empty<'a> {
    msg: &'a str,
}
impl<'a> Empty<'a> {
    pub fn new(msg: &'a str) -> Self {
        Self { msg }
    }
}
impl Widget for Empty<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Block::default()
            .borders(Borders::ALL)
            .title(Line::from(format!(" {} ", self.msg)).centered())
            .render(area, buf);
    }
}

pub struct PanelFrame;
impl PanelFrame {
    pub fn render(
        title: Option<Block<'static>>,
        area: Rect,
        buf: &mut Buffer,
        mut child: impl FnMut(Rect, &mut Buffer),
    ) {
        if let Some(block) = title {
            let inner = block.inner(area);
            block.render(area, buf);
            child(inner, buf);
        } else {
            child(area, buf);
        }
    }
}

pub struct EnginePanel;
impl<C: Chromosome> UiPanel<C> for EnginePanel {
    fn id(&self) -> PanelId {
        PanelId::Engine
    }

    fn title(&self, state: &AppState<C>) -> Option<Line<'static>> {
        let engine_state = if state.is_engine_running() {
            if state.is_engine_paused() {
                " Paused ".fg(Color::Yellow).bold()
            } else {
                " Running ".fg(Color::LightGreen).bold()
            }
        } else {
            " Complete ".fg(Color::Red).bold()
        };

        Some(Line::from(engine_state).alignment(Alignment::Center))
    }

    fn render(&self, state: &mut AppState<C>, area: Rect, buf: &mut Buffer) {
        PanelFrame::render(self.block(state), area, buf, |inner, buf| {
            EngineBaseWidget::new(state).render(inner, buf);
        });
    }
}

pub struct FiltersPanel;
impl<C: Chromosome> UiPanel<C> for FiltersPanel {
    fn id(&self) -> PanelId {
        PanelId::Filters
    }

    fn title(&self, _state: &AppState<C>) -> Option<Line<'static>> {
        Some(Line::from(" Filters ").alignment(Alignment::Center))
    }

    fn render(&self, state: &mut AppState<C>, area: Rect, buf: &mut Buffer) {
        PanelFrame::render(self.block(state), area, buf, |inner, buf| {
            FilterWidget::new(state).render(inner, buf);
        });
    }
}

pub struct MetricsPanel;
impl<C: Chromosome> UiPanel<C> for MetricsPanel {
    fn id(&self) -> PanelId {
        PanelId::Metrics
    }

    fn block(&self, _: &AppState<C>) -> Option<Block<'static>> {
        Some(
            Block::bordered()
                .title_bottom(help_text_minimal())
                .title_top(" Metrics ")
                .border_type(BorderType::Rounded)
                .title_alignment(Alignment::Center),
        )
    }

    fn render(&self, mut state: &mut AppState<C>, area: Rect, buf: &mut Buffer) {
        PanelFrame::render(self.block(state), area, buf, |inner, buf| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Fill(1)])
                .split(inner);

            let titles = ["Stats", "Time"]
                .into_iter()
                .map(|t| Span::styled(format!(" {t} "), Style::default().fg(Color::White)));

            let index = match state.metrics_tab {
                MetricsTab::Stats => 0,
                MetricsTab::Time => 1,
            };

            Tabs::new(titles)
                .select(index)
                .padding(" ", " ")
                .divider(" ")
                .highlight_style(styles::selected_item_style())
                .bold()
                .render(chunks[0], buf);

            match state.metrics_tab {
                MetricsTab::Time => TimeTableWidget::new(&mut state).render(chunks[1], buf),
                MetricsTab::Stats => StatsTableWidget::new(&mut state).render(chunks[1], buf),
            }
        });
    }
}

pub struct FitnessPanel;
impl<C: Chromosome> UiPanel<C> for FitnessPanel {
    fn id(&self) -> PanelId {
        PanelId::Fitness
    }

    fn title(&self, _state: &AppState<C>) -> Option<Line<'static>> {
        None
    }

    fn render(&self, state: &mut AppState<C>, area: Rect, buf: &mut Buffer) {
        PanelFrame::render(self.block(state), area, buf, |inner, buf| {
            if state.objective_state.objective.is_single() {
                let chart_state = state.chart_state();
                let charts = if state.display_mini_chart_mean() {
                    vec![
                        chart_state.fitness_chart(),
                        chart_state.fitness_mean_chart(),
                    ]
                } else {
                    vec![chart_state.fitness_chart()]
                };

                ChartWidget::from(charts).render(inner, buf);
            } else {
                ParetoPagingWidget::new(&state).render(inner, buf);
            }
        });
    }
}

pub struct HelpPanel;
impl<C: Chromosome> UiPanel<C> for HelpPanel {
    fn id(&self) -> PanelId {
        PanelId::Help
    }

    fn title(&self, _state: &AppState<C>) -> Option<Line<'static>> {
        Some(Line::from(" Help ").alignment(Alignment::Center))
    }

    fn render(&self, state: &mut AppState<C>, area: Rect, buf: &mut Buffer) {
        PanelFrame::render(self.block(state), area, buf, |inner, buf| {
            let body = Paragraph::new(help_text())
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true });
            body.render(inner, buf);
        });
    }
}

fn help_text() -> Text<'static> {
    Text::from(vec![
        Line::from(vec![Span::styled(
            "Controls",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("General"),
        Line::from("  q           Quit UI"),
        Line::from("  ? / H       Toggle this help"),
        Line::from("  p           Pause / Resume engine"),
        Line::from("  n           Step one epoch (stays paused)"),
        Line::from(""),
        Line::from("Navigation"),
        Line::from("  j / Down    Move selection down"),
        Line::from("  k / Up      Move selection up"),
        Line::from("  h / Left    Previous metrics tab"),
        Line::from("  l / Right   Next metrics tab"),
        Line::from(""),
        Line::from("Charts / Objective pairs"),
        Line::from("  [ / ]       Prev / next objective-pair page"),
        Line::from("  + / -       Expand / shrink objective pairs"),
        Line::from("  c           Toggle mini chart"),
        Line::from("  m           Toggle mini chart mean"),
        Line::from(""),
        Line::from("Filters"),
        Line::from("  f           Toggle tag filters panel"),
        Line::from("  Enter       Toggle tag selection"),
        Line::from("  Esc         Clear tag filters"),
        Line::from("  0-9         Select filter by index"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press Esc to close",
            Style::default().add_modifier(Modifier::DIM),
        )]),
    ])
}

fn help_text_minimal<'a>() -> Line<'a> {
    Line::from(vec![
        "[j/k]".fg(Color::LightGreen).bold(),
        Span::from(" navigate, "),
        "[◄ ►/h/l]".fg(Color::LightGreen).bold(),
        Span::from(" change tab, "),
        "[f]".fg(Color::LightGreen).bold(),
        Span::from(" toggle filters, "),
        "[?/H]".fg(Color::LightGreen).bold(),
        Span::from(" help "),
    ])
    .centered()
}
