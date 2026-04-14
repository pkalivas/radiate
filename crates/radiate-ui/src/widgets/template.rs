use radiate_engines::Chromosome;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Span,
    widgets::{StatefulWidget, Tabs, Widget},
};

use crate::{
    state::{AppState, PanelId, TabId},
    widgets::{
        DistributionTableWidget, EngineDashboardPanelWidget, EngineStatusPanelWidget,
        FitnessChartPanelWidget, MetricDetailPanelWidget, StatsTableWidget, TabComponent,
        TimeTableWidget,
        components::{SpeciesPieChartComponent, TimePieChartComponent},
        tables::SpeciesTableWidget,
    },
};

pub trait RenderNode<S> {
    fn render(&self, area: Rect, buf: &mut ratatui::buffer::Buffer, state: &mut S);
}

pub enum LayoutNode {
    Horizontal {
        constraints: Vec<Constraint>,
        children: Vec<LayoutNode>,
    },
    Vertical {
        constraints: Vec<Constraint>,
        children: Vec<LayoutNode>,
    },
    Tabbed {
        id: TabId,
        tabs: Vec<&'static str>,
        children: Vec<LayoutNode>,
    },
    Widget(PanelId),
}

impl LayoutNode {
    pub fn draw<C: Chromosome>(&self, area: Rect, buf: &mut Buffer, state: &mut AppState<C>) {
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
            LayoutNode::Tabbed { id, tabs, children } => {
                let active_tab_idx = state.active_tab_index(id);

                let areas = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Fill(1)])
                    .split(area);

                TabComponent::from(tabs)
                    .select(active_tab_idx)
                    .render(areas[0], buf);

                if let Some(active_child) = children.get(active_tab_idx) {
                    active_child.draw(areas[1], buf, state);
                }
            }

            LayoutNode::Widget(panel_id) => match panel_id {
                PanelId::EngineStatus => EngineStatusPanelWidget::new().render(area, buf, state),
                PanelId::FitnessChart => FitnessChartPanelWidget::new().render(area, buf, state),

                PanelId::TimeTable => TimeTableWidget::new().render(area, buf, state),
                PanelId::StatsTable => StatsTableWidget::new().render(area, buf, state),
                PanelId::DistTable => DistributionTableWidget::new().render(area, buf, state),
                PanelId::SpeciesTable => SpeciesTableWidget::new().render(area, buf, state),

                PanelId::MetricDetail => MetricDetailPanelWidget::new().render(area, buf, state),

                PanelId::SpeciesPieChart => {
                    SpeciesPieChartComponent::new().render(area, buf, state)
                }

                PanelId::TimePieChart => TimePieChartComponent::new().render(area, buf, state),

                _ => {}
            },
        }
    }
}

impl Default for LayoutNode {
    fn default() -> Self {
        LayoutNode::Vertical {
            constraints: vec![Constraint::Percentage(30), Constraint::Fill(1)],
            children: vec![
                LayoutNode::Horizontal {
                    constraints: vec![Constraint::Percentage(25), Constraint::Fill(1)],
                    children: vec![
                        LayoutNode::Widget(PanelId::EngineStatus),
                        LayoutNode::Widget(PanelId::FitnessChart),
                    ],
                },
                LayoutNode::Tabbed {
                    id: TabId::Dashboard,
                    tabs: vec!["Stats", "Time", "Distribution", "Species"],
                    children: vec![
                        LayoutNode::Horizontal {
                            constraints: vec![Constraint::Fill(1), Constraint::Percentage(20)],
                            children: vec![
                                LayoutNode::Widget(PanelId::StatsTable),
                                LayoutNode::Widget(PanelId::MetricDetail),
                            ],
                        },
                        LayoutNode::Horizontal {
                            constraints: vec![
                                Constraint::Percentage(30),
                                Constraint::Fill(1),
                                Constraint::Percentage(20),
                            ],
                            children: vec![
                                LayoutNode::Widget(PanelId::TimePieChart),
                                LayoutNode::Widget(PanelId::TimeTable),
                                LayoutNode::Widget(PanelId::MetricDetail),
                            ],
                        },
                        LayoutNode::Widget(PanelId::DistTable),
                        LayoutNode::Horizontal {
                            constraints: vec![
                                Constraint::Fill(1),
                                Constraint::Percentage(25),
                                Constraint::Percentage(25),
                            ],
                            children: vec![
                                LayoutNode::Widget(PanelId::SpeciesTable),
                                LayoutNode::Widget(PanelId::SpeciesPieChart),
                            ],
                        },
                    ],
                },
            ],
        }
    }
}
