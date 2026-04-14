use crate::state::{AppState, PanelId};
use crate::widgets::{
    EngineDashboardPanelWidget, EngineStatusPanelWidget, FitnessChartPanelWidget, HelpWidget,
    LayoutNode, MetricModalWidget, ModalWidget,
};
use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use radiate_engines::stats::TagType;
use radiate_engines::{
    Chromosome, CommandChannel, EngineControl, Front, MetricSet, Objective, Phenotype, Score,
    SpeciesSnapshot,
};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Paragraph, StatefulWidget, Widget};
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
    EpochComplete(
        usize,
        MetricSet,
        Score,
        Option<Front<Phenotype<C>>>,
        Option<Vec<SpeciesSnapshot>>,
    ),
}

pub(crate) struct App<C>
where
    C: Chromosome,
{
    control: EngineControl,
    channel: CommandChannel<InputEvent<C>>,
    state: AppState<C>,
}

impl<C> App<C>
where
    C: Chromosome,
{
    pub fn new(render_interval: Duration, control: EngineControl) -> Self {
        Self {
            control,
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
                    if self.state.search_state.active {
                        self.handle_search_key_event(key_event);
                    } else {
                        self.handle_key_event(key_event.code);
                    }
                }
                _ => {}
            },
            InputEvent::EngineStart(objective) => {
                self.handle_engine_start(objective);
            }
            InputEvent::EngineStop => self.state.running.engine = false,
            InputEvent::EpochComplete(index, metrics, score, front, species_snapshots) => {
                self.handle_engine_epoch(index, metrics, score, front, species_snapshots);
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

    fn handle_search_key_event(&mut self, key: KeyEvent) {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => {
                if self.state.search_state.query.is_empty() {
                    self.state.clear_search();
                } else {
                    self.state.stop_search();
                }
            }
            (KeyCode::Enter, _) => self.state.stop_search(),
            (KeyCode::Backspace, _) => self.state.pop_search_char(),

            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.state.stop_search();
                self.state.clear_search();
            }
            (KeyCode::Char(c), _) => self.state.push_search_char(c),
            _ => {}
        }
    }

    fn handle_key_event(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('/') => self.state.start_search(),

            KeyCode::Char('q') => {
                self.control.stop();
                self.state.running.ui = false
            }

            KeyCode::Char('?') | KeyCode::Char('H') => self.state.toggle_help(),

            KeyCode::Char('f') => self.state.toggle_show_tag_filters(),

            KeyCode::Down | KeyCode::Char('j') => self.state.move_selection_down(),
            KeyCode::Up | KeyCode::Char('k') => self.state.move_selection_up(),

            KeyCode::Char(']') => self.state.next_objective_pair_page(),
            KeyCode::Char('[') => self.state.previous_objective_pair_page(),
            KeyCode::Char('+') => self.state.expand_objective_pairs(),
            KeyCode::Char('-') => self.state.shrink_objective_pairs(),

            KeyCode::Right | KeyCode::Char('l') => self.state.next_tab(),
            KeyCode::Left | KeyCode::Char('h') => self.state.previous_tab(),

            KeyCode::Char('p') => {
                let paused = self.control.toggle_pause();
                self.state.running.paused = paused;
            }
            KeyCode::Char('n') => {
                self.control.step_once();
                self.state.running.paused = true;
            }

            KeyCode::Esc => self.state.clear_filters(),
            KeyCode::Enter => self.state.toggle_tag_filter_selection(),

            KeyCode::Char(c) => {
                if let Some(digit) = c.to_digit(10) {
                    self.state.set_objective_index(digit as usize);
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
        mut species_snapshots: Option<Vec<SpeciesSnapshot>>,
    ) {
        // for metric in metrics.iter() {
        //     self.state.chart_state.update_from_metric(metric.1);
        // }

        // self.state.metrics = metrics;
        self.state.score = score;
        self.state.index = index;

        self.state.update_metrics(metrics);
        self.state.update_species(species_snapshots);

        // self.state.filter_state.all_tags = self
        //     .state
        //     .metrics
        //     .tags()
        //     .filter(|tag| {
        //         *tag != TagType::Statistic && *tag != TagType::Time && *tag != TagType::Distribution
        //     })
        //     .collect();

        // self.state.filter_state.all_tags.sort();

        if let Some(front) = front {
            self.state.front = Some(front);

            let total = super::widgets::num_pairs(self.state.objective_state.objective.dims());
            if total > 0 {
                self.state.objective_state.chart_start_index =
                    self.state.objective_state.chart_start_index.min(total - 1);
            } else {
                self.state.objective_state.chart_start_index = 0;
            }
        }

        // if let Some(species_snapshots) = &mut species_snapshots {
        //     species_snapshots.sort_unstable_by(|a, b| a.id.0.cmp(&b.id.0));
        // }

        // self.state.species = species_snapshots;
    }

    pub fn handle_engine_start(&mut self, objective: Objective) {
        self.state.running.engine = true;
        self.state.objective_state.objective = objective.clone();
        if objective.dims() == 2 {
            self.state.objective_state.charts_visible = 1;
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
            Style::default()
                .bg(crate::styles::ALT_BG_COLOR)
                .fg(crate::styles::TEXT_FG_COLOR),
        );

        let [top, bottom] =
            Layout::vertical([Constraint::Percentage(2), Constraint::Fill(1)]).areas(area);

        Paragraph::new(format!("{:?}", self.state.display.focus_panel)).render(top, buf);

        let template = LayoutNode::default();

        template.draw(bottom, buf, &mut self.state);

        // let [top, bottom] =
        //     Layout::vertical([Constraint::Percentage(30), Constraint::Fill(1)]).areas(area);

        // let [summary, fitness] =
        //     Layout::horizontal([Constraint::Percentage(25), Constraint::Fill(1)]).areas(top);

        // EngineStatusPanelWidget::new().render(summary, buf, &mut self.state);
        // FitnessChartPanelWidget::new().render(fitness, buf, &mut self.state);
        // EngineDashboardPanelWidget::new().render(bottom, buf, &mut self.state);

        if let Some(panel) = self.state.display.modal_panel {
            match panel {
                PanelId::Help => ModalWidget::new(HelpWidget).render(bottom, buf),
                PanelId::MetricModal => {
                    ModalWidget::new(MetricModalWidget::new()).render(bottom, buf, &mut self.state);
                }
                _ => {}
            }
        }
    }
}
