use crate::defaults::TEXT_FG_COLOR;
use crate::state::{AppState, MetricsTab};
use crate::widgets::filter::FilterWidget;
use crate::widgets::summary::EngineBaseWidget;
use crate::widgets::{MetricsTabWidget, ParetoFrontTemp, ParetoFrontWidget, kth_pair, num_pairs};
use color_eyre::Result;
use crossterm::event::{Event, KeyCode};
use radiate_engines::stats::{TagKind, metric_tags};
use radiate_engines::{
    Chromosome, CommandChannel, Front, MetricSet, Objective, Phenotype, Score, metric_names,
};
use ratatui::buffer::Buffer;
use ratatui::style::Style;
use ratatui::widgets::Widget;
use ratatui::{Terminal, backend::CrosstermBackend};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::palette::material,
};
use std::sync::{Arc, mpsc};
use std::{
    io,
    time::{Duration, Instant},
};

pub(crate) enum InputEvent<C>
where
    C: Chromosome,
{
    Crossterm(Event),
    EngineStart(Objective),
    EngineStop,
    EpochComplete(usize, MetricSet, Score, Option<Front<Phenotype<C>>>),
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
            InputEvent::EngineStart(objective) => {
                self.handle_engine_start(objective);
            }
            InputEvent::EngineStop => self.state.running.engine = false,
            InputEvent::EpochComplete(index, metrics, score, front) => {
                self.handle_engine_epoch(index, metrics, score, front);
                let now = Instant::now();
                if let Some(last) = self.state.last_render {
                    let elapsed = now.duration_since(last);
                    if elapsed < self.state.render_interval() {
                        return Ok(false);
                    }
                }

                self.state.last_render = Some(now);
            }
        }

        Ok(true)
    }

    fn handle_key_event(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.state.running.ui = false,

            KeyCode::Char('f') => self.state.toggle_show_tag_filters(),

            KeyCode::Char('c') => self.state.toggle_mini_chart(),
            KeyCode::Char('m') => self.state.toggle_mini_chart_mean(),

            KeyCode::Down | KeyCode::Char('j') => self.state.move_selection_down(),
            KeyCode::Up | KeyCode::Char('k') => self.state.move_selection_up(),

            KeyCode::Char(']') => self.state.next_objective_pair_page(),
            KeyCode::Char('[') => self.state.previous_objective_pair_page(),
            KeyCode::Char('+') => self.state.expand_objective_pairs(),
            KeyCode::Char('-') => self.state.shrink_objective_pairs(),

            KeyCode::Right | KeyCode::Char('l') => self.state.next_metrics_tab(),
            KeyCode::Left | KeyCode::Char('h') => self.state.previous_metrics_tab(),

            KeyCode::Esc => self.state.clear_tag_filters(),
            KeyCode::Enter => self.state.toggle_tag_filter_selection(),

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
        front: Option<Front<Phenotype<C>>>,
    ) {
        let charts = self.state.charts_mut();
        charts
            .fitness_chart_mut()
            .update_last_value(index as f64, score.as_f32() as f64);

        if let Some(dist) = metrics
            .get(metric_names::SCORES)
            .and_then(|m| m.statistic())
        {
            charts
                .fitness_chart_mut()
                .update_mean_value(index as f64, dist.mean() as f64);
        }

        for metric in metrics.iter() {
            self.state.chart_state.update_from_metric(metric.1);
        }

        self.state.metrics = metrics;
        self.state.score = score;
        self.state.index = index;

        self.state.filter_state.all_tags = self
            .state
            .metrics
            .tags()
            .filter(|tag| *tag != TagKind::Statistic && *tag != TagKind::Time)
            .collect();
        self.state.filter_state.all_tags.sort();
        if let Some(front) = front {
            self.state.front = Some(front);

            let total =
                super::widgets::num_pairs(self.state.objective_state.objective.dimensions());
            if total > 0 {
                self.state.objective_state.chart_start_index =
                    self.state.objective_state.chart_start_index.min(total - 1);
            } else {
                self.state.objective_state.chart_start_index = 0;
            }
        }
    }

    pub fn handle_engine_start(&mut self, objective: Objective) {
        self.state.running.engine = true;
        self.state.objective_state.objective = objective.clone();
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
        let [engine, fitness] =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Fill(1)]).areas(top);

        EngineBaseWidget::new(&self.state).render(engine, buf);

        if self.state.objective_state.objective.is_single() {
            if self.state.display_mini_chart_mean() {
                self.state.charts().fitness_chart().render(fitness, buf);
            } else {
                self.state
                    .charts()
                    .fitness_chart()
                    .value_chart()
                    .render(fitness, buf);
            }
        } else {
            ParetoFrontTemp::new(&self.state).render(fitness, buf);
        }

        if self.state.display.show_tag_filters {
            let [filter, tabs] =
                Layout::horizontal([Constraint::Length(20), Constraint::Fill(1)]).areas(bottom);
            FilterWidget::new(&mut self.state).render(filter, buf);
            MetricsTabWidget::new(&mut self.state).render(tabs, buf);
        } else {
            let [inner] = Layout::horizontal([Constraint::Fill(1)]).areas(bottom);
            MetricsTabWidget::new(&mut self.state).render(inner, buf);
        };
    }
}
