use crate::{
    state::{AppState, MetricChartType},
    widgets::{
        AppWidget, EngineStatusPanelWidget, FnWidget, MetricDetailPanelWidget, MetricTableWidget,
        Panel, ParetoPagingWidget, SearchBarWidget, TabComponent,
        components::{SpeciesPieChartComponent, SpeciesSparklineComponent, TimePieChartComponent},
        panels::{MetricLineChartWidget, tables::SpeciesTableWidget},
    },
};
use radiate_engines::{Chromosome, metric_names};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
};

pub struct TabNode<C: Chromosome> {
    pub title: &'static str,
    pub condition: fn(&AppState<C>) -> bool,
    pub content: LayoutNode<C>,
}

pub enum LayoutNode<C: Chromosome> {
    Horizontal {
        constraints: Vec<Constraint>,
        children: Vec<LayoutNode<C>>,
    },
    Vertical {
        constraints: Vec<Constraint>,
        children: Vec<LayoutNode<C>>,
    },
    Tabbed {
        children: Vec<TabNode<C>>,
    },
    Widget(fn(Rect, &mut Buffer, &mut AppState<C>)),
}

impl<C: Chromosome> LayoutNode<C> {
    pub fn draw(&self, area: Rect, buf: &mut Buffer, state: &mut AppState<C>) {
        match self {
            LayoutNode::Horizontal {
                constraints,
                children,
            } => {
                let areas = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(constraints)
                    .split(area);

                for (child, &child_area) in children.iter().zip(areas.iter()) {
                    child.draw(child_area, buf, state);
                }
            }
            LayoutNode::Vertical {
                constraints,
                children,
            } => {
                let areas = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(constraints)
                    .split(area);

                for (child, &child_area) in children.iter().zip(areas.iter()) {
                    child.draw(child_area, buf, state);
                }
            }
            LayoutNode::Tabbed { children } => {
                let active_tab_idx = state.nav.dashboard_tab_index();

                let areas = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Fill(1)])
                    .split(area);

                let titles = children
                    .iter()
                    .filter(|t| (t.condition)(state))
                    .map(|t| t.title)
                    .collect::<Vec<_>>();

                let select = children[..active_tab_idx]
                    .iter()
                    .filter(|t| (t.condition)(state))
                    .count();

                Panel::new(FnWidget::new(|area, buf| {
                    TabComponent::from(titles).select(select).render(area, buf);
                }))
                .render_inside_block(true)
                .render(areas[0], buf);

                if let Some(active_child) = children.get(active_tab_idx) {
                    active_child.content.draw(areas[1], buf, state);
                }
            }
            LayoutNode::Widget(render) => render(area, buf, state),
        }
    }
}

impl<C: Chromosome> Default for LayoutNode<C> {
    fn default() -> Self {
        use LayoutNode::*;

        Vertical {
            constraints: vec![Constraint::Percentage(30), Constraint::Fill(1)],
            children: vec![
                Horizontal {
                    constraints: vec![Constraint::Percentage(25), Constraint::Fill(1)],
                    children: vec![
                        Widget(|a, b, s| EngineStatusPanelWidget::new().render(a, b, s)),
                        Widget(|a, b, s| {
                            if s.evo.pareto.objective.is_single() {
                                MetricLineChartWidget::new(
                                    metric_names::BEST_SCORES,
                                    MetricChartType::Last,
                                )
                                .render(a, b, s)
                            } else {
                                ParetoPagingWidget::new(s).render(a, b);
                            }
                        }),
                    ],
                },
                Vertical {
                    constraints: vec![Constraint::Fill(1), Constraint::Length(3)],
                    children: vec![
                        Tabbed {
                            children: vec![
                                TabNode {
                                    title: "Stats",
                                    condition: |_| true,
                                    content: Horizontal {
                                        constraints: vec![
                                            Constraint::Fill(1),
                                            Constraint::Percentage(30),
                                            Constraint::Percentage(20),
                                        ],
                                        children: vec![
                                            Widget(|a, b, s| {
                                                MetricTableWidget::stats().render(a, b, s)
                                            }),
                                            Widget(|a, b, s| {
                                                MetricLineChartWidget::default().render(a, b, s)
                                            }),
                                            Widget(|a, b, s| {
                                                MetricDetailPanelWidget.render(a, b, s)
                                            }),
                                        ],
                                    },
                                },
                                TabNode {
                                    title: "Time",
                                    condition: |_| true,
                                    content: Horizontal {
                                        constraints: vec![
                                            Constraint::Fill(1),
                                            Constraint::Percentage(30),
                                            Constraint::Percentage(20),
                                        ],
                                        children: vec![
                                            Widget(|a, b, s| {
                                                MetricTableWidget::time().render(a, b, s)
                                            }),
                                            Widget(|a, b, s| {
                                                TimePieChartComponent::new().render(a, b, s)
                                            }),
                                            Widget(|a, b, s| {
                                                MetricDetailPanelWidget.render(a, b, s)
                                            }),
                                        ],
                                    },
                                },
                                TabNode {
                                    title: "Distribution",
                                    condition: |_| true,
                                    content: Horizontal {
                                        constraints: vec![
                                            Constraint::Fill(1),
                                            Constraint::Percentage(30),
                                            Constraint::Percentage(20),
                                        ],
                                        children: vec![
                                            Widget(|a, b, s| {
                                                MetricTableWidget::distribution().render(a, b, s)
                                            }),
                                            Widget(|a, b, s| {
                                                MetricLineChartWidget::default().render(a, b, s)
                                            }),
                                            Widget(|a, b, s| {
                                                MetricDetailPanelWidget.render(a, b, s)
                                            }),
                                        ],
                                    },
                                },
                                TabNode {
                                    title: "Species",
                                    condition: |s| s.evo.has_species(),
                                    content: Horizontal {
                                        constraints: vec![
                                            Constraint::Fill(1),
                                            Constraint::Percentage(25),
                                            Constraint::Percentage(25),
                                        ],
                                        children: vec![
                                            Widget(|a, b, s| {
                                                SpeciesTableWidget::new().render(a, b, s)
                                            }),
                                            Widget(|a, b, s| {
                                                SpeciesSparklineComponent::new().render(a, b, s)
                                            }),
                                            Widget(|a, b, s| {
                                                SpeciesPieChartComponent::new().render(a, b, s)
                                            }),
                                        ],
                                    },
                                },
                            ],
                        },
                        Horizontal {
                            constraints: vec![Constraint::Fill(1)],
                            children: vec![Widget(|a, b, s| SearchBarWidget::new(s).render(a, b))],
                        },
                    ],
                },
            ],
        }
    }
}
