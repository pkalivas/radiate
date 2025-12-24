use crate::state::AppState;
use crate::{EnginePanel, FiltersPanel, FitnessPanel, HelpPanel, MetricsPanel, PanelId, UiPanel};
use radiate_engines::Chromosome;
use ratatui::style::Style;
use ratatui::widgets::StatefulWidget;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Clear, Widget},
};
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

pub static MAIN_TEMPLATE: LazyLock<UiNode> = LazyLock::new(|| UiNode::Overlay {
    base: Box::new(UiNode::Split {
        dir: Direction::Vertical,
        constraints: vec![Constraint::Percentage(30), Constraint::Fill(1)],
        children: vec![
            UiNode::Split {
                dir: Direction::Horizontal,
                constraints: vec![Constraint::Percentage(30), Constraint::Fill(1)],
                children: vec![
                    UiNode::Panel(PanelId::Engine),
                    UiNode::Panel(PanelId::Fitness),
                ],
            },
            UiNode::IfActive {
                panel: PanelId::Filters,
                active_child: Box::new(UiNode::Split {
                    dir: Direction::Horizontal,
                    constraints: vec![Constraint::Length(20), Constraint::Fill(1)],
                    children: vec![
                        UiNode::Panel(PanelId::Filters),
                        UiNode::Panel(PanelId::Metrics),
                    ],
                }),
                inactive_child: Some(Box::new(UiNode::Panel(PanelId::Metrics))),
            },
        ],
    }),
    modal: PanelId::Help,
    modal_rect: (70, 80),
});

#[derive(Clone)]
pub enum UiNode {
    Panel(PanelId),
    Split {
        dir: Direction,
        constraints: Vec<Constraint>,
        children: Vec<UiNode>,
    },
    IfActive {
        panel: PanelId,
        active_child: Box<UiNode>,
        inactive_child: Option<Box<UiNode>>,
    },
    Overlay {
        base: Box<UiNode>,
        modal: PanelId,
        modal_rect: (u16, u16),
    },
}

pub struct AppUi<C: Chromosome> {
    pub model: UiModel,
    pub registry: PanelRegistry<C>,
}

impl<C: Chromosome> AppUi<C> {
    pub fn new() -> Self {
        let mut model = UiModel::default();

        model.set_active(PanelId::Engine, true);
        model.set_active(PanelId::Metrics, true);
        model.set_active(PanelId::Fitness, true);
        model.set_active(PanelId::Filters, false);
        model.set_active(PanelId::Help, false);

        let registry = PanelRegistry::new()
            .register(EnginePanel)
            .register(MetricsPanel)
            .register(FiltersPanel)
            .register(FitnessPanel)
            .register(HelpPanel);

        Self { model, registry }
    }

    pub fn set_panel_active(&mut self, id: PanelId, on: bool) {
        self.model.set_active(id, on);
    }

    pub fn set_modal(&mut self, id: Option<PanelId>) {
        if let Some(id) = id {
            self.model.set_active(id, true);
        }

        self.model.modal = id;
    }
}

impl<C: Chromosome> StatefulWidget for &AppUi<C> {
    type State = AppState<C>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        buf.set_style(
            area,
            Style::default()
                .bg(crate::styles::ALT_BG_COLOR)
                .fg(crate::styles::TEXT_FG_COLOR),
        );

        let ui = Ui {
            root: &MAIN_TEMPLATE,
            registry: &self.registry,
        };

        ui.render_node(&ui.root, state, &self.model, area, buf);
    }
}

pub struct PanelRegistry<C: Chromosome> {
    panels: HashMap<PanelId, Box<dyn UiPanel<C>>>,
}

impl<C: Chromosome> PanelRegistry<C> {
    pub fn new() -> Self {
        Self {
            panels: HashMap::new(),
        }
    }

    pub fn register(mut self, p: impl UiPanel<C> + 'static) -> Self {
        self.panels.insert(p.id(), Box::new(p));
        self
    }

    pub fn get(&self, id: PanelId) -> Option<&dyn UiPanel<C>> {
        self.panels.get(&id).map(|b| &**b)
    }
}

#[derive(Default)]
pub struct UiModel {
    pub active: HashSet<PanelId>,
    pub modal: Option<PanelId>,
}

impl UiModel {
    pub fn set_active(&mut self, id: PanelId, on: bool) {
        if on {
            self.active.insert(id);
        } else {
            self.active.remove(&id);
        }
    }
}

pub struct Ui<'a, C: Chromosome> {
    pub root: &'a UiNode,
    pub registry: &'a PanelRegistry<C>,
}

impl<'a, C: Chromosome> Ui<'a, C> {
    pub fn new(root: &'a UiNode, registry: &'a PanelRegistry<C>) -> Self {
        Self { root, registry }
    }

    fn render_node(
        &self,
        node: &UiNode,
        state: &mut AppState<C>,
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
            UiNode::IfActive {
                panel,
                active_child: child,
                inactive_child,
            } => {
                if model.active.contains(panel) {
                    self.render_node(child, state, model, area, buf);
                } else if let Some(inactive_child) = inactive_child {
                    self.render_node(inactive_child, state, model, area, buf);
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
            UiNode::Overlay {
                base,
                modal,
                modal_rect,
            } => {
                self.render_node(base, state, model, area, buf);

                if model.modal == Some(*modal) {
                    let r = centered_rect(modal_rect.0, modal_rect.1, area);
                    Clear.render(r, buf);
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
