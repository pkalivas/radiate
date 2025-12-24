use crate::state::AppState;
use crate::widgets::root::RootWidget;
use color_eyre::Result;
use crossterm::event::{Event, KeyCode};
use radiate_engines::stats::TagKind;
use radiate_engines::{
    Chromosome, CommandChannel, EngineControl, Front, MetricSet, Objective, Phenotype, Score,
    metric_names,
};
use ratatui::buffer::Buffer;
use ratatui::style::Style;
use ratatui::widgets::Widget;
use ratatui::{Terminal, backend::CrosstermBackend};
use ratatui::{layout::Rect, style::palette::material};
use std::sync::{Arc, LazyLock, mpsc};
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
    control: EngineControl,
}

impl<C> App<C>
where
    C: Chromosome,
{
    pub fn new(render_interval: Duration, control: EngineControl) -> Self {
        Self {
            channel: CommandChannel::new(),
            control,
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

            KeyCode::Char('f') => self.state.toggle_show_tag_filters(),

            KeyCode::Char('c') => self.state.toggle_mini_chart(),
            KeyCode::Char('m') => self.state.toggle_mini_chart_mean(),
            KeyCode::Char('?') | KeyCode::Char('H') => {
                self.state.toggle_help();
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
        buf.set_style(
            area,
            Style::default()
                .bg(material::GRAY.c900)
                .fg(crate::styles::TEXT_FG_COLOR),
        );

        // MAIN_TEMPLATE

        // RootWidget::new(&mut self.state).render(area, buf);
    }
}

use ratatui::layout::{Constraint, Direction, Layout};
use std::collections::{HashMap, HashSet};

pub static MAIN_TEMPLATE: LazyLock<Template> = LazyLock::new(|| Template {
    root: UiNode::Overlay {
        base: Box::new(UiNode::Split {
            dir: Direction::Vertical,
            constraints: vec![Constraint::Percentage(30), Constraint::Fill(1)],
            children: vec![
                UiNode::Split {
                    dir: Direction::Horizontal,
                    constraints: vec![Constraint::Percentage(30), Constraint::Fill(1)],
                    children: vec![
                        UiNode::Panel(PanelId::Engine),
                        UiNode::Tabs {
                            id: "top_right",
                            children: vec![
                                UiNode::Panel(PanelId::Fitness),
                                UiNode::Panel(PanelId::Pareto),
                            ],
                        },
                    ],
                },
                UiNode::Split {
                    dir: Direction::Horizontal,
                    constraints: vec![Constraint::Length(20), Constraint::Fill(1)],
                    children: vec![
                        UiNode::IfActive {
                            panel: PanelId::Filters,
                            child: Box::new(UiNode::Panel(PanelId::Filters)),
                        },
                        UiNode::Panel(PanelId::Metrics),
                    ],
                },
            ],
        }),
        modal: PanelId::Help,
        modal_rect: (70, 80),
    },
});

// ---------- Panel registry ----------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PanelId {
    Engine,
    Fitness,
    Pareto,
    Metrics,
    Filters,
    Help,
}

pub trait Panel<C: Chromosome>: Send + Sync {
    fn id(&self) -> PanelId;
    fn title(&self) -> &'static str {
        ""
    }
    fn render(&self, state: &AppState<C>, area: Rect, buf: &mut Buffer);
}

// Helper: store trait objects
pub struct PanelRegistry<C: Chromosome> {
    panels: HashMap<PanelId, Box<dyn Panel<C>>>,
}

impl<C: Chromosome> PanelRegistry<C> {
    pub fn new() -> Self {
        Self {
            panels: HashMap::new(),
        }
    }

    pub fn register(mut self, p: impl Panel<C> + 'static) -> Self {
        self.panels.insert(p.id(), Box::new(p));
        self
    }

    pub fn get(&self, id: PanelId) -> Option<&dyn Panel<C>> {
        self.panels.get(&id).map(|b| &**b)
    }
}

// ---------- Template tree ----------

#[derive(Clone)]
pub enum UiNode {
    /// Render a panel (if active)
    Panel(PanelId),

    /// Split area into children
    Split {
        dir: Direction,
        constraints: Vec<Constraint>,
        children: Vec<UiNode>,
    },

    /// Tabs: render only selected child
    Tabs {
        id: &'static str,      // key for tab state
        children: Vec<UiNode>, // typically panels
    },

    /// Render child only if panel is active
    IfActive { panel: PanelId, child: Box<UiNode> },

    /// Overlay: render base then optional modal panel (centered)
    Overlay {
        base: Box<UiNode>,
        modal: PanelId,         // what to render when active
        modal_rect: (u16, u16), // percent (x,y)
    },
}

pub struct Template {
    pub root: UiNode,
}

// ---------- Dynamic UI model ----------

pub struct UiModel {
    pub active: HashSet<PanelId>,
    pub tabs: HashMap<&'static str, usize>,
    pub modal: Option<PanelId>,
}

impl UiModel {
    pub fn new() -> Self {
        Self {
            active: HashSet::new(),
            tabs: HashMap::new(),
            modal: None,
        }
    }

    pub fn set_active(&mut self, id: PanelId, on: bool) {
        if on {
            self.active.insert(id);
        } else {
            self.active.remove(&id);
        }
    }

    pub fn toggle(&mut self, id: PanelId) {
        let on = !self.active.contains(&id);
        self.set_active(id, on);
    }

    pub fn tab_next(&mut self, key: &'static str, len: usize) {
        let i = self.tabs.get(key).copied().unwrap_or(0);
        self.tabs.insert(key, (i + 1) % len.max(1));
    }

    pub fn tab_prev(&mut self, key: &'static str, len: usize) {
        let i = self.tabs.get(key).copied().unwrap_or(0);
        self.tabs
            .insert(key, i.saturating_sub(1).min(len.saturating_sub(1)));
    }
}

// ---------- Renderer ----------

pub struct Ui<'a, C: Chromosome> {
    pub template: &'a Template,         // static
    pub registry: &'a PanelRegistry<C>, // static
}

impl<'a, C: Chromosome> Ui<'a, C> {
    pub fn render(&self, state: &AppState<C>, model: &UiModel, area: Rect, buf: &mut Buffer) {
        self.render_node(&self.template.root, state, model, area, buf);
    }

    fn render_node(
        &self,
        node: &UiNode,
        state: &AppState<C>,
        model: &UiModel,
        area: Rect,
        buf: &mut Buffer,
    ) {
        match node {
            UiNode::Panel(id) => {
                if model.active.contains(id) {
                    if let Some(p) = self.registry.get(*id) {
                        p.render(state, area, buf);
                    }
                }
            }

            UiNode::IfActive { panel, child } => {
                if model.active.contains(panel) {
                    self.render_node(child, state, model, area, buf);
                }
            }

            UiNode::Split {
                dir,
                constraints,
                children,
            } => {
                let chunks = Layout::default()
                    .direction(*dir)
                    .constraints(constraints.clone())
                    .split(area);

                for (i, child) in children.iter().enumerate() {
                    if let Some(r) = chunks.get(i).copied() {
                        self.render_node(child, state, model, r, buf);
                    }
                }
            }

            UiNode::Tabs { id, children } => {
                let idx = model
                    .tabs
                    .get(id)
                    .copied()
                    .unwrap_or(0)
                    .min(children.len().saturating_sub(1));
                if let Some(child) = children.get(idx) {
                    self.render_node(child, state, model, area, buf);
                }
            }

            UiNode::Overlay {
                base,
                modal,
                modal_rect,
            } => {
                // render base always
                self.render_node(base, state, model, area, buf);

                // render modal if chosen
                if model.modal == Some(*modal) {
                    let r = centered_rect(modal_rect.0, modal_rect.1, area);
                    // you likely want Clear + Block here inside the panel itself or wrap with a Modal panel
                    if let Some(p) = self.registry.get(*modal) {
                        p.render(state, r, buf);
                    }
                }
            }
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
