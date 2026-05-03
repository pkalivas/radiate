use crate::{
    state::AppState,
    widgets::{
        EngineStatusPanelWidget, FitnessChartPanelWidget, FnWidget, MetricDetailPanelWidget,
        MetricTableWidget, Panel, SearchBarWidget, TabComponent,
        components::{SpeciesPieChartComponent, SpeciesSparklineComponent, TimePieChartComponent},
        panels::tables::SpeciesTableWidget,
    },
};
use radiate_engines::Chromosome;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{StatefulWidget, Widget},
};

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
        tabs: Vec<&'static str>,
        children: Vec<LayoutNode<C>>,
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

                for (child, &child_area) in children.iter().zip(areas.into_iter()) {
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

                for (child, &child_area) in children.iter().zip(areas.into_iter()) {
                    child.draw(child_area, buf, state);
                }
            }
            LayoutNode::Tabbed { tabs, children } => {
                let active_tab_idx = state.nav.dashboard_tab_index();

                let areas = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Fill(1)])
                    .split(area);

                Panel::new(FnWidget::new(|area, buf| {
                    TabComponent::from(tabs.clone())
                        .select(active_tab_idx)
                        .render(area, buf);
                }))
                .render_inside_block(true)
                .render(areas[0], buf);

                if let Some(active_child) = children.get(active_tab_idx) {
                    active_child.draw(areas[1], buf, state);
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
                        Widget(|a, b, s| FitnessChartPanelWidget::new().render(a, b, s)),
                    ],
                },
                Vertical {
                    constraints: vec![Constraint::Fill(1), Constraint::Length(3)],
                    children: vec![
                        Tabbed {
                            tabs: vec!["Stats", "Time", "Distribution", "Species"],
                            children: vec![
                                Horizontal {
                                    constraints: vec![
                                        Constraint::Fill(1),
                                        Constraint::Percentage(15),
                                    ],
                                    children: vec![
                                        Widget(|a, b, s| {
                                            MetricTableWidget::stats().render(a, b, s)
                                        }),
                                        Widget(|a, b, s| {
                                            MetricDetailPanelWidget::new().render(a, b, s)
                                        }),
                                    ],
                                },
                                Horizontal {
                                    constraints: vec![
                                        Constraint::Fill(1),
                                        Constraint::Percentage(30),
                                        Constraint::Percentage(20),
                                    ],
                                    children: vec![
                                        Widget(|a, b, s| MetricTableWidget::time().render(a, b, s)),
                                        Widget(|a, b, s| {
                                            TimePieChartComponent::new().render(a, b, s)
                                        }),
                                        Widget(|a, b, s| {
                                            MetricDetailPanelWidget::new().render(a, b, s)
                                        }),
                                    ],
                                },
                                Horizontal {
                                    constraints: vec![
                                        Constraint::Fill(1),
                                        Constraint::Percentage(20),
                                    ],
                                    children: vec![
                                        Widget(|a, b, s| {
                                            MetricTableWidget::distribution().render(a, b, s)
                                        }),
                                        Widget(|a, b, s| {
                                            MetricDetailPanelWidget::new().render(a, b, s)
                                        }),
                                    ],
                                },
                                Horizontal {
                                    constraints: vec![
                                        Constraint::Fill(1),
                                        Constraint::Percentage(25),
                                        Constraint::Percentage(25),
                                    ],
                                    children: vec![
                                        Widget(|a, b, s| SpeciesTableWidget::new().render(a, b, s)),
                                        Widget(|a, b, s| {
                                            SpeciesSparklineComponent::new().render(a, b, s)
                                        }),
                                        Widget(|a, b, s| {
                                            SpeciesPieChartComponent::new().render(a, b, s)
                                        }),
                                    ],
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
