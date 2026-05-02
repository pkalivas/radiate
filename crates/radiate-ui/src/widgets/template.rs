use crate::{
    state::{AppState, PanelId, TabId},
    widgets::{
        DistributionTableWidget, EngineStatusPanelWidget, FitnessChartPanelWidget, FnWidget,
        MetricDetailPanelWidget, Panel, SearchBarWidget, StatsTableWidget, TabComponent,
        TimeTableWidget,
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
                PanelId::SpeciesSparkline => {
                    SpeciesSparklineComponent::new().render(area, buf, state)
                }
                PanelId::Search => SearchBarWidget::new(state).render(area, buf),
                _ => {}
            },
        }
    }
}

impl Default for LayoutNode {
    fn default() -> Self {
        use LayoutNode::*;

        Vertical {
            constraints: vec![Constraint::Percentage(30), Constraint::Fill(1)],
            children: vec![
                Horizontal {
                    constraints: vec![Constraint::Percentage(25), Constraint::Fill(1)],
                    children: vec![Widget(PanelId::EngineStatus), Widget(PanelId::FitnessChart)],
                },
                Vertical {
                    constraints: vec![Constraint::Fill(1), Constraint::Length(3)],
                    children: vec![
                        Tabbed {
                            id: TabId::Dashboard,
                            tabs: vec!["Stats", "Time", "Distribution", "Species"],
                            children: vec![
                                Horizontal {
                                    constraints: vec![
                                        Constraint::Fill(1),
                                        Constraint::Percentage(15),
                                    ],
                                    children: vec![
                                        Widget(PanelId::StatsTable),
                                        Widget(PanelId::MetricDetail),
                                    ],
                                },
                                Horizontal {
                                    constraints: vec![
                                        Constraint::Fill(1),
                                        Constraint::Percentage(30),
                                        Constraint::Percentage(20),
                                    ],
                                    children: vec![
                                        Widget(PanelId::TimeTable),
                                        Widget(PanelId::TimePieChart),
                                        Widget(PanelId::MetricDetail),
                                    ],
                                },
                                Horizontal {
                                    constraints: vec![
                                        Constraint::Fill(1),
                                        Constraint::Percentage(20),
                                    ],
                                    children: vec![
                                        Widget(PanelId::DistTable),
                                        Widget(PanelId::MetricDetail),
                                    ],
                                },
                                Horizontal {
                                    constraints: vec![
                                        Constraint::Fill(1),
                                        Constraint::Percentage(25),
                                        Constraint::Percentage(25),
                                    ],
                                    children: vec![
                                        Widget(PanelId::SpeciesTable),
                                        Widget(PanelId::SpeciesSparkline),
                                        Widget(PanelId::SpeciesPieChart),
                                    ],
                                },
                            ],
                        },
                        Horizontal {
                            constraints: vec![Constraint::Fill(1)],
                            children: vec![Widget(PanelId::Search)],
                        },
                    ],
                },
            ],
        }
    }
}
