use crate::keys::{self, Action};
use crate::state::{AppState, Dir, RunState, UiMode};
use crate::widgets::{AppWidget, HelpPanelWidget, LayoutNode, MetricModalWidget, ModalWidget};
use color_eyre::Result;
use crossbeam::channel;
use crossterm::event::{Event, KeyCode, KeyEvent};
use radiate_engines::{
    Chromosome, CommandChannel, Ecosystem, Front, GenerationSnapshot, Improvement, LogEvent,
    MetricSet, Objective, Phenotype, Score, StreamSnapshot, Subscription, ThreadSync,
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

pub const DEFAULT_RENDER_INTERVAL: Duration = Duration::from_millis(100);

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
    Log(LogEvent),
    Improvement(ImprovementEvent),
    Stream(StreamSnapshot),
    Subscription(Subscription),
}

pub(crate) struct App<C>
where
    C: Chromosome,
{
    sync: ThreadSync,
    channel: CommandChannel<InputEvent<C>>,
    state: AppState<C>,
    layout: LayoutNode<C>,
}

impl<C> App<C>
where
    C: Chromosome + Debug + Clone,
{
    pub fn new(sync: ThreadSync) -> Self {
        let pause_flag = sync.pause_flag();
        Self {
            sync,
            channel: CommandChannel::new(),
            state: AppState {
                run: RunState {
                    render_interval: DEFAULT_RENDER_INTERVAL,
                    pause_flag: Some(pause_flag),
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
        self.state.run.event_count += 1;
        match self.channel.next()? {
            InputEvent::Crossterm(event) => {
                if let Event::Key(key) = event
                    && let Some(action) = keys::action(self.state.nav.mode, &key)
                {
                    self.apply(action, key);
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
            InputEvent::Log(event) => {
                self.handle_log(event);
                return self.throttle();
            }
            InputEvent::Improvement(event) => {
                self.handle_improvement(event);
                return self.throttle();
            }
            InputEvent::Stream(snapshot) => {
                self.handle_stream(snapshot);
                return self.throttle();
            }
            InputEvent::Subscription(subscription) => {
                self.handle_subscription(subscription);
                return self.throttle();
            }
        }

        Ok(true)
    }

    fn throttle(&mut self) -> Result<bool> {
        if !self.state.run.started {
            return Ok(false);
        }

        let now = Instant::now();

        if self.sync.is_paused() {
            self.state.run.last_render = Some(now);
            return Ok(true);
        }

        if let Some(last) = self.state.run.last_render {
            let elapsed = now.duration_since(last);
            if elapsed < self.state.run.render_interval {
                return Ok(false);
            }
        }

        self.state.run.last_render = Some(now);
        Ok(true)
    }

    fn apply(&mut self, action: Action, key: KeyEvent) {
        let state = &mut self.state;
        match action {
            Action::Quit => {
                self.sync.stop();
                for subscription in state.run.subscriptions.drain(..) {
                    subscription.unsubscribe();
                }
                state.run.ui = false;
            }
            Action::ToggleHelp => state.nav.toggle_help(),
            Action::TogglePause => {
                self.sync.toggle_pause();
            }
            Action::Step => {
                self.sync.step_once();
            }

            Action::Select(motion) => state.move_selection(motion),
            Action::Tab(dir) => state.nav.step_tab(dir, &state.evo),
            Action::Cycle(dir) => state.cycle(dir),
            Action::Inspect(motion) => state.inspect.scroll(motion),

            Action::ToggleModal => state.nav.toggle_metric_modal(),
            Action::ToggleLogScale => state.nav.chart_view.toggle_scale(),
            Action::CycleHistory => state.nav.chart_view.next_history(),
            Action::ObjectivePage(Dir::Next) => state.evo.next_objective_pair_page(),
            Action::ObjectivePage(Dir::Prev) => state.evo.previous_objective_pair_page(),
            Action::ObjectivePairs(Dir::Next) => state.evo.expand_objective_pairs(),
            Action::ObjectivePairs(Dir::Prev) => state.evo.shrink_objective_pairs(),
            Action::Objective => {
                if let KeyCode::Char(c) = key.code
                    && let Some(digit) = c.to_digit(10)
                {
                    state.evo.set_objective_index(digit as usize);
                }
            }

            Action::OpenSearch => state.nav.open_search(),
            Action::CloseSearch => state.nav.close_search(),
            Action::CancelSearch => {
                state.nav.close_search();
                state.nav.clear_search();
            }
            Action::ClearFilter => state.nav.clear_search_query(),
            Action::Type => {
                if let KeyCode::Char(c) = key.code {
                    state.nav.push_search_char(c);
                }
            }
            Action::Backspace => state.nav.pop_search_char(),
        }
    }

    fn handle_engine_epoch(&mut self, event: GenerationEvent<C>) {
        self.state.evo.update_score(event.score);
        self.state.evo.update_index(event.index);
        self.state.evo.update_ecosystem(event.ecosystem);
        self.state.evo.update_metrics(event.metrics);

        self.state.nav.ensure_tab_available(&self.state.evo);
    }

    pub fn handle_improvement(&mut self, event: ImprovementEvent) {
        self.state.evo.update_improvement(event);
    }

    pub fn handle_log(&mut self, event: LogEvent) {
        self.state.evo.push_event_log_entry(event.0, event.1);
    }

    pub fn handle_stream(&mut self, event: StreamSnapshot) {
        self.state.stream.update(event);
    }

    pub fn handle_subscription(&mut self, event: Subscription) {
        self.state.run.subscriptions.push(event);
    }

    pub fn handle_engine_start(&mut self, front: Arc<RwLock<Front<Phenotype<C>>>>) {
        let objectives = front.read().unwrap().objective().clone();

        self.state.run.engine = true;
        self.state.run.started = true;
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
