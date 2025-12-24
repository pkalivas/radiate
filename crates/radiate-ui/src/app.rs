use crate::state::AppState;
use crate::{AppUi, PanelId};
use color_eyre::Result;
use crossterm::event::{Event, KeyCode};
use radiate_engines::stats::TagKind;
use radiate_engines::{
    Chromosome, CommandChannel, EngineControl, Front, MetricSet, Objective, Phenotype, Score,
    metric_names,
};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{StatefulWidget, Widget};
use ratatui::{Terminal, backend::CrosstermBackend};
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
    control: EngineControl,
    channel: CommandChannel<InputEvent<C>>,
    state: AppState<C>,
    ui: AppUi<C>,
}

impl<C> App<C>
where
    C: Chromosome,
{
    pub fn new(render_interval: Duration, control: EngineControl) -> Self {
        Self {
            control,
            channel: CommandChannel::new(),
            ui: AppUi::new(),
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
            KeyCode::Char('q') => {
                self.control.stop();
                self.state.running.ui = false
            }

            KeyCode::Char('f') => {
                let active = self.state.toggle_show_tag_filters();
                self.ui.set_panel_active(PanelId::Filters, active);
            }

            KeyCode::Char('c') => self.state.toggle_mini_chart(),
            KeyCode::Char('m') => self.state.toggle_mini_chart_mean(),
            KeyCode::Char('?') | KeyCode::Char('H') => {
                let show = self.state.toggle_help();
                self.ui.set_modal(match show {
                    true => Some(PanelId::Help),
                    false => None,
                });
            }

            KeyCode::Down | KeyCode::Char('j') => self.state.move_selection_down(),
            KeyCode::Up | KeyCode::Char('k') => self.state.move_selection_up(),

            KeyCode::Char(']') => self.state.next_objective_pair_page(),
            KeyCode::Char('[') => self.state.previous_objective_pair_page(),
            KeyCode::Char('+') => self.state.expand_objective_pairs(),
            KeyCode::Char('-') => self.state.shrink_objective_pairs(),

            KeyCode::Right | KeyCode::Char('l') => self.state.next_metrics_tab(),
            KeyCode::Left | KeyCode::Char('h') => self.state.previous_metrics_tab(),

            KeyCode::Char('p') => {
                let paused = self.control.toggle_pause();
                self.state.running.paused = paused;
            }
            KeyCode::Char('n') => {
                self.control.step_once();
                self.state.running.paused = true;
            }

            KeyCode::Esc => {
                self.state.clear_filters();
                self.ui.set_panel_active(PanelId::Filters, false);
                self.ui.set_modal(None);
            }
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
        let charts = self.state.chart_state_mut();
        charts.fitness_chart_mut().push(score.as_f32() as f64);

        if let Some(dist) = metrics
            .get(metric_names::SCORES)
            .and_then(|m| m.statistic())
        {
            charts.fitness_mean_chart_mut().push(dist.mean() as f64);
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
        StatefulWidget::render(&self.ui, area, buf, &mut self.state);
    }
}
