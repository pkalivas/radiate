use crate::defaults::TEXT_FG_COLOR;
use crate::state::{AppState, MetricsTab};
use crate::widgets::MetricsTabWidget;
use crate::widgets::filter::FilterWidget;
use crate::widgets::summary::EngineBaseWidget;
use color_eyre::Result;
use crossterm::event::{Event, KeyCode};
use radiate_engines::stats::metric_tags;
use radiate_engines::{
    Chromosome, CommandChannel, Front, MetricSet, Objective, Phenotype, Score, metric_names,
};
use ratatui::buffer::Buffer;
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols;
use ratatui::text::Line;
use ratatui::widgets::{Axis, Block, Chart, Dataset, GraphType, Widget};
use ratatui::{Terminal, backend::CrosstermBackend};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::palette::material,
};
use std::sync::{Arc, RwLock, mpsc};
use std::{
    io,
    time::{Duration, Instant},
};

pub(crate) enum InputEvent<C>
where
    C: Chromosome,
{
    Crossterm(Event),
    EngineStart(Objective, Option<Arc<RwLock<Front<Phenotype<C>>>>>),
    EngineStop,
    EpochComplete(usize, MetricSet, Score, Objective),
}

pub(crate) struct App<C>
where
    C: Chromosome,
{
    state: AppState<C>,
    channel: CommandChannel<InputEvent<C>>,
}

impl<C> App<C>
where
    C: Chromosome,
{
    pub fn new(render_interval: Duration) -> Self {
        Self {
            channel: CommandChannel::new(),
            state: AppState {
                render_interval,
                ..Default::default()
            },
        }
    }

    pub fn dispatcher(&self) -> Arc<mpsc::Sender<InputEvent<C>>> {
        self.channel.dispatcher()
    }

    pub fn run(mut self, mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        while self.state.running.ui {
            if self.throttle_next()? {
                terminal.draw(|f| {
                    self.render(f.area(), f.buffer_mut());
                })?;
            }
        }

        Ok(())
    }

    fn throttle_next(&mut self) -> Result<bool> {
        match self.channel.next()? {
            InputEvent::Crossterm(event) => match event {
                Event::Key(key_event) => {
                    self.handle_key_event(key_event.code);
                }
                _ => {}
            },
            InputEvent::EngineStart(objective, front) => {
                self.handle_engine_start(objective, front);
            }
            InputEvent::EngineStop => self.state.running.engine = false,
            InputEvent::EpochComplete(index, metrics, score, objective) => {
                self.handle_engine_epoch(index, metrics, score, objective);
                let now = Instant::now();
                if let Some(last) = self.state.last_render {
                    let elapsed = now.duration_since(last);
                    if elapsed < self.state.render_interval() {
                        return Ok(false);
                    }
                }

                self.state.last_render = Some(Instant::now());
            }
        }

        Ok(true)
    }

    fn handle_key_event(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.state.running.ui = false,

            KeyCode::Char('f') => self.state.toggle_show_tag_filters(),

            KeyCode::Char('t') => self.state.set_metrics_tab(MetricsTab::Time),
            KeyCode::Char('s') => self.state.set_metrics_tab(MetricsTab::Stats),
            KeyCode::Char('d') => self.state.set_metrics_tab(MetricsTab::Distributions),

            KeyCode::Char('c') => self.state.toggle_mini_chart(),
            KeyCode::Char('m') => self.state.toggle_mini_chart_mean(),

            KeyCode::Down | KeyCode::Char('j') => self.state.move_selection_down(),
            KeyCode::Up | KeyCode::Char('k') => self.state.move_selection_up(),

            KeyCode::Char(c) => {
                if let Some(digit) = c.to_digit(10) {
                    self.state.set_tag_filter_by_index(digit as usize);
                }
            }

            _ => {}
        }
    }

    fn handle_engine_epoch(
        &mut self,
        index: usize,
        metrics: MetricSet,
        score: Score,
        objective: Objective,
    ) {
        let charts = self.state.charts_mut();
        charts
            .fitness_chart_mut()
            .update_last_value(index as f64, score.as_f32() as f64);

        if let Some(dist) = metrics
            .get(metric_names::SCORES)
            .and_then(|m| m.distribution())
        {
            charts
                .fitness_chart_mut()
                .update_mean_value(index as f64, dist.mean() as f64);
        }

        for metric in metrics.iter() {
            let key = metric.0;
            let chart = charts.get_or_create_chart(key);

            chart.update(metric.1);
        }

        self.state.metrics = metrics;
        self.state.score = score;
        self.state.index = index;
        self.state.objective = objective;

        self.state.all_tags = self
            .state
            .metrics
            .tags()
            .filter(|tag| {
                tag.0 != metric_tags::STATISTIC
                    && tag.0 != metric_tags::DISTRIBUTION
                    && tag.0 != metric_tags::TIME
            })
            .cloned()
            .collect();
        self.state.all_tags.sort();
    }

    pub fn handle_engine_start(
        &mut self,
        objective: Objective,
        front: Option<Arc<RwLock<Front<Phenotype<C>>>>>,
    ) {
        self.state.running.engine = true;
        self.state.objective = objective;
        if let Some(front) = front {
            self.state.front = Some(front);
        }
    }
}

impl<C> Widget for &mut App<C>
where
    C: Chromosome,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_style(
            area,
            Style::default().bg(material::GRAY.c900).fg(TEXT_FG_COLOR),
        );

        let [top, bottom] =
            Layout::vertical([Constraint::Percentage(30), Constraint::Fill(1)]).areas(area);

        let [engine, fitness, score_dist] = Layout::horizontal([
            Constraint::Percentage(30),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .areas(top);

        EngineBaseWidget::new(&self.state).render(engine, buf);

        if self.state.objective.is_single() {
            self.state.charts().fitness_chart().render(fitness, buf);
            if let Some(dist_chart) = self.state.get_chart_by_key(metric_names::SCORES) {
                dist_chart.value_chart().render(score_dist, buf);
            }
        } else {
            ParetoFrontWidget::new(&self.state, 0, 1).render(fitness, buf);
            ParetoFrontWidget::new(&self.state, 1, 2).render(score_dist, buf);
        }

        if self.state.display.show_tag_filters {
            let [filter, tabs] =
                Layout::horizontal([Constraint::Length(3 + 20), Constraint::Fill(1)]).areas(bottom);
            FilterWidget::new(&self.state).render(filter, buf);
            MetricsTabWidget::new(&mut self.state).render(tabs, buf);
        } else {
            let [inner] = Layout::horizontal([Constraint::Fill(1)]).areas(bottom);
            MetricsTabWidget::new(&mut self.state).render(inner, buf);
        };
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
            Some(f) if !f.read().unwrap().is_empty() => f,
            _ => {
                Block::bordered()
                    .title(" Pareto Front (no data) ")
                    .render(area, buf);
                return;
            }
        };

        let mut points: Vec<(f64, f64)> = Vec::new();
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        let reader = front.read().unwrap();

        for p in reader.values().iter() {
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
                .title(" Pareto Front (no valid scores) ")
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
            .style(Style::default().fg(Color::Cyan))
            .data(&points);

        let chart = Chart::new(vec![dataset])
            .block(Block::bordered().title(" Pareto Front (obj0 vs obj1) "))
            .x_axis(
                Axis::default()
                    .title(format!("Objective {}", self.obj_one))
                    .style(Style::default().gray())
                    .bounds([min_x, max_x]),
            )
            .y_axis(
                Axis::default()
                    .title(format!("Objective {}", self.obj_two))
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
