use crate::state::{AppState, RunState, UiMode, nav::DashboardTab};
use crate::widgets::{AppWidget, HelpPanelWidget, LayoutNode, MetricModalWidget, ModalWidget};
use color_eyre::Result;
use crossbeam::channel;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use radiate_engines::{
    Chromosome, CommandChannel, Ecosystem, Front, GenerationSnapshot, Improvement, MetricSet,
    Objective, Phenotype, Score, StreamSnapshot, Subscription, ThreadSync, events::LogLevel,
};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Widget;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
};
use std::{
    io,
    time::{Duration, Instant},
};

pub struct GenerationEvent<C>
where
    C: Chromosome,
{
    pub index: usize,
    pub metrics: MetricSet,
    pub score: Score,
    pub ecosystem: Ecosystem<C>,
}

impl<C, T> From<&GenerationSnapshot<C, T>> for GenerationEvent<C>
where
    C: Chromosome + Clone,
    T: Clone + Send + Sync + 'static,
{
    fn from(snapshot: &GenerationSnapshot<C, T>) -> Self {
        Self {
            index: snapshot.generation.index(),
            metrics: snapshot.generation.metrics().clone(),
            score: snapshot.generation.score().clone(),
            ecosystem: snapshot.generation.ecosystem().clone(),
        }
    }
}

pub struct ImprovementEvent {
    pub index: usize,
    pub score: Score,
    pub previous_score: Option<Score>,
    pub objective: Objective,
}

impl<T> From<&Improvement<T>> for ImprovementEvent
where
    T: Clone + Send + Sync + 'static,
{
    fn from(improvement: &Improvement<T>) -> Self {
        Self {
            index: improvement.index,
            score: improvement.score.clone(),
            previous_score: improvement.previous_score.clone(),
            objective: improvement.objective.clone(),
        }
    }
}

// Clippy assumes that the `CrossTerm(Event)` event is the high-frequency event and
// `EpochComplete` is the low-frequency variant - but it is exactly the opposite. `EpochComplete` events can happen hundreds, if
// not thousands, of times per second; boxing it would add a heap allocation on every generation tick,
// which is far worse than the size overhead.
#[allow(clippy::large_enum_variant)]
pub(crate) enum InputEvent<C>
where
    C: Chromosome,
{
    Crossterm(Event),
    EngineStart(Arc<RwLock<Front<Phenotype<C>>>>),
    EngineStop,
    EpochComplete(GenerationEvent<C>),
    Log(LogLevel, String),
    Improvement(ImprovementEvent),
    Stream(StreamSnapshot),
    Subscription(Subscription),
}

pub(crate) struct App<C>
where
    C: Chromosome,
{
    control: ThreadSync,
    channel: CommandChannel<InputEvent<C>>,
    state: AppState<C>,
    layout: LayoutNode<C>,
}

impl<C> App<C>
where
    C: Chromosome + Debug + Clone,
{
    pub fn new(render_interval: Duration, control: ThreadSync) -> Self {
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

    pub fn dispatcher(&self) -> channel::Sender<InputEvent<C>> {
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
            InputEvent::Crossterm(event) => {
                if let Event::Key(key_event) = event {
                    match self.state.nav.mode {
                        UiMode::Search => self.handle_search_event(key_event),
                        UiMode::Help => self.handle_help_event(key_event),
                        UiMode::MetricModal => self.handle_metric_modal_event(key_event),
                        UiMode::Dashboard => self.handle_dashboard_event(key_event.code),
                    }
                }
            }
            InputEvent::EngineStart(front) => {
                self.handle_engine_start(front);
            }
            InputEvent::EngineStop => self.state.run.engine = false,
            InputEvent::EpochComplete(event) => {
                self.handle_engine_epoch(event);
                return self.throttle();
            }
            InputEvent::Log(level, message) => {
                self.state.run.event_count += 1;
                self.state.evo.push_event_log_entry(level, message);
                return self.throttle();
            }
            InputEvent::Improvement(event) => {
                self.handle_improvement(event);
                return self.throttle();
            }
            InputEvent::Stream(snapshot) => {
                self.state.stream.update(snapshot);
                return self.throttle();
            }
            InputEvent::Subscription(subscription) => {
                self.state.run.subscriptions.push(subscription);
            }
        }

        Ok(true)
    }

    fn throttle(&mut self) -> Result<bool> {
        let now = Instant::now();
        if let Some(last) = self.state.run.last_render {
            let elapsed = now.duration_since(last);
            if elapsed < self.state.run.render_interval {
                return Ok(false);
            }
        }

        self.state.run.last_render = Some(now);
        Ok(true)
    }

    fn handle_metric_modal_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                self.state.nav.toggle_metric_modal()
            }

            KeyCode::Right | KeyCode::Tab => self.state.next_chart_view(),
            KeyCode::Left | KeyCode::BackTab => self.state.prev_chart_view(),

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
            (KeyCode::Esc, _) | (KeyCode::Enter, _) | (KeyCode::Char('/'), _) => {
                self.state.nav.close_search()
            }
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
                for subscription in &self.state.run.subscriptions {
                    subscription.unsubscribe();
                }
                self.state.run.subscriptions.clear();
                self.state.run.ui = false
            }

            KeyCode::Char('?') | KeyCode::Char('H') => self.state.nav.toggle_help(),

            KeyCode::Down | KeyCode::Char('j') => self.state.move_selection_down(),
            KeyCode::Up | KeyCode::Char('k') => self.state.move_selection_up(),
            KeyCode::PageDown | KeyCode::Char('d') => self.state.move_selection_page_down(),
            KeyCode::PageUp | KeyCode::Char('u') => self.state.move_selection_page_up(),
            KeyCode::Home | KeyCode::Char('g') => self.state.move_selection_to_top(),
            KeyCode::End | KeyCode::Char('G') => self.state.move_selection_to_bottom(),

            KeyCode::Char(']') => self.state.evo.next_objective_pair_page(),
            KeyCode::Char('[') => self.state.evo.previous_objective_pair_page(),
            KeyCode::Char('+') => self.state.evo.expand_objective_pairs(),
            KeyCode::Char('-') => self.state.evo.shrink_objective_pairs(),

            KeyCode::Right | KeyCode::Char('l') => self
                .state
                .nav
                .next_tab(self.state.evo.has_species(), self.state.evo.is_multi()),
            KeyCode::Left | KeyCode::Char('h') => self
                .state
                .nav
                .previous_tab(self.state.evo.has_species(), self.state.evo.is_multi()),

            KeyCode::Tab => match self.state.nav.dashboard_tab {
                DashboardTab::Population => self.state.next_chromosome(),
                _ => self.state.next_chart_view(),
            },
            KeyCode::BackTab => match self.state.nav.dashboard_tab {
                DashboardTab::Population => self.state.previous_chromosome(),
                _ => self.state.prev_chart_view(),
            },

            KeyCode::Char('J') => self.state.inspect.scroll_by(1),
            KeyCode::Char('K') => self.state.inspect.scroll_by(-1),
            KeyCode::Char('D') => self.state.inspect.scroll_page(1),
            KeyCode::Char('U') => self.state.inspect.scroll_page(-1),

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

    fn handle_engine_epoch(&mut self, event: GenerationEvent<C>) {
        self.state.evo.update_score(event.score);
        self.state.evo.update_index(event.index);
        self.state.evo.update_ecosystem(event.ecosystem);
        self.state.evo.update_metrics(event.metrics);

        self.state
            .nav
            .ensure_tab_available(self.state.evo.has_species(), self.state.evo.is_multi());
        self.state.run.event_count += 1;
    }

    pub fn handle_improvement(&mut self, event: ImprovementEvent) {
        self.state.evo.update_improvement(event);
        self.state.run.event_count += 1;
    }

    pub fn handle_engine_start(&mut self, front: Arc<RwLock<Front<Phenotype<C>>>>) {
        let objectives = front.read().unwrap().objective().clone();

        self.state.run.engine = true;
        self.state.evo.pareto.objective = objectives.clone();
        if objectives.dims() == 2 {
            self.state.evo.pareto.charts_visible = 1;
        }

        self.state.evo.front = Arc::clone(&front);

        let total = super::widgets::num_pairs(self.state.evo.pareto.objective.dims());
        if total > 0 {
            self.state.evo.pareto.chart_start_index =
                self.state.evo.pareto.chart_start_index.min(total - 1);
        } else {
            self.state.evo.pareto.chart_start_index = 0;
        }

        self.state.run.event_count += 1;
    }
}

impl<C> Widget for &mut App<C>
where
    C: Chromosome + Debug,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_style(
            area,
            Style::default()
                .bg(crate::styles::ALT_BG_COLOR)
                .fg(crate::styles::TEXT_FG_COLOR),
        );

        self.state.run.render_count += 1;
        self.layout.draw(area, buf, &mut self.state);

        match self.state.nav.mode {
            UiMode::Help => ModalWidget::new(HelpPanelWidget).render(area, buf),
            UiMode::MetricModal => {
                ModalWidget::new(MetricModalWidget).render(area, buf, &mut self.state);
            }
            _ => {}
        }
    }
}
