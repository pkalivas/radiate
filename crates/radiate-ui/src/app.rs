use crate::state::{AppState, RunState, UiMode};
use crate::widgets::{HelpPanelWidget, LayoutNode, MetricModalWidget, ModalWidget};
use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use radiate_engines::{
    Chromosome, CommandChannel, EngineControl, Front, MetricSet, Objective, Phenotype, Score,
    SpeciesSnapshot,
};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
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
    layout: LayoutNode<C>,
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
                run: RunState {
                    render_interval,
                    ..Default::default()
                },
                ..Default::default()
            },
            layout: LayoutNode::default(),
        }
    }

    pub fn dispatcher(&self) -> Arc<mpsc::Sender<InputEvent<C>>> {
        self.channel.dispatcher()
    }

    pub fn run(mut self, mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        while self.state.run.ui {
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
                Event::Key(key_event) => match self.state.nav.mode {
                    UiMode::Search => self.handle_search_event(key_event),
                    UiMode::Help => self.handle_help_event(key_event),
                    UiMode::MetricModal => self.handle_metric_modal_event(key_event),
                    UiMode::Dashboard => self.handle_dashboard_event(key_event.code),
                },
                _ => {}
            },
            InputEvent::EngineStart(objective) => {
                self.handle_engine_start(objective);
            }
            InputEvent::EngineStop => self.state.run.engine = false,
            InputEvent::EpochComplete(index, metrics, score, front, species_snapshots) => {
                self.handle_engine_epoch(index, metrics, score, front, species_snapshots);
                let now = Instant::now();
                if let Some(last) = self.state.run.last_render {
                    let elapsed = now.duration_since(last);
                    if elapsed < self.state.run.render_interval {
                        return Ok(false);
                    }
                }

                self.state.run.last_render = Some(now);
            }
        }

        Ok(true)
    }

    fn handle_metric_modal_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => self.state.nav.toggle_metric_modal(),

            KeyCode::Right | KeyCode::Char('l') => self.state.nav.next_tab(),
            KeyCode::Left | KeyCode::Char('h') => self.state.nav.previous_tab(),

            KeyCode::Char('p') => {
                let paused = self.control.toggle_pause();
                self.state.run.paused = paused;
            }
            KeyCode::Char('n') => {
                self.control.step_once();
                self.state.run.paused = true;
            }

            _ => {}
        }
    }

    fn handle_help_event(&mut self, key: KeyEvent) {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) | (KeyCode::Char('H'), _) | (KeyCode::Char('?'), _) => {
                self.state.nav.toggle_help();
            }
            _ => {}
        }
    }

    fn handle_search_event(&mut self, key: KeyEvent) {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) | (KeyCode::Enter, _) => self.state.nav.close_search(),
            (KeyCode::Backspace, _) => self.state.nav.pop_search_char(),

            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.state.nav.close_search();
                self.state.nav.clear_search();
            }
            (KeyCode::Char(c), _) => self.state.nav.push_search_char(c),
            _ => {}
        }
    }

    fn handle_dashboard_event(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('/') => self.state.nav.open_search(),

            KeyCode::Char('q') => {
                self.control.stop();
                self.state.run.ui = false
            }

            KeyCode::Char('?') | KeyCode::Char('H') => self.state.nav.toggle_help(),

            KeyCode::Down | KeyCode::Char('j') => self.state.move_selection_down(),
            KeyCode::Up | KeyCode::Char('k') => self.state.move_selection_up(),

            KeyCode::Char(']') => self.state.evo.next_objective_pair_page(),
            KeyCode::Char('[') => self.state.evo.previous_objective_pair_page(),
            KeyCode::Char('+') => self.state.evo.expand_objective_pairs(),
            KeyCode::Char('-') => self.state.evo.shrink_objective_pairs(),

            KeyCode::Right | KeyCode::Char('l') => self.state.nav.next_tab(),
            KeyCode::Left | KeyCode::Char('h') => self.state.nav.previous_tab(),

            KeyCode::Char('p') => {
                let paused = self.control.toggle_pause();
                self.state.run.paused = paused;
            }
            KeyCode::Char('n') => {
                self.control.step_once();
                self.state.run.paused = true;
            }

            KeyCode::Esc => self.state.nav.clear_search_query(),
            KeyCode::Enter => self.state.nav.toggle_metric_modal(),

            KeyCode::Char(c) => {
                if let Some(digit) = c.to_digit(10) {
                    self.state.evo.set_objective_index(digit as usize);
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
        species_snapshots: Option<Vec<SpeciesSnapshot>>,
    ) {
        self.state.evo.score = score;
        self.state.evo.index = index;

        self.state.evo.update_metrics(metrics);
        self.state.evo.update_species(species_snapshots);

        if let Some(front) = front {
            self.state.evo.front = Some(front);

            let total = super::widgets::num_pairs(self.state.evo.pareto.objective.dims());
            if total > 0 {
                self.state.evo.pareto.chart_start_index =
                    self.state.evo.pareto.chart_start_index.min(total - 1);
            } else {
                self.state.evo.pareto.chart_start_index = 0;
            }
        }
    }

    pub fn handle_engine_start(&mut self, objective: Objective) {
        self.state.run.engine = true;
        self.state.evo.pareto.objective = objective.clone();
        if objective.dims() == 2 {
            self.state.evo.pareto.charts_visible = 1;
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

        self.layout.draw(area, buf, &mut self.state);

        match self.state.nav.mode {
            UiMode::Help => ModalWidget::new(HelpPanelWidget).render(area, buf),
            UiMode::MetricModal => {
                ModalWidget::new(MetricModalWidget::new()).render(area, buf, &mut self.state);
            }
            _ => {}
        }
    }
}
