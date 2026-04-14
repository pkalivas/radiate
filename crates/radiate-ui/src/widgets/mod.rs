mod chart;
mod components;
mod help;
mod modal;
mod panel;
mod panels;
mod pareto;
mod search;
mod tables;

pub use chart::LineChartWidget;
pub use components::TabComponent;
pub use help::HelpWidget;
pub use modal::ModalWidget;
pub use panel::{FnWidget, Panel};
pub use panels::{
    EngineDashboardPanelWidget, EngineStatusPanelWidget, FitnessChartPanelWidget,
    MetricDetailPanelWidget, MetricModalWidget, TagsPanelWidget,
};
pub use pareto::{ParetoPagingWidget, num_pairs};
pub use search::SearchBarWidget;
pub use tables::{DistributionTableWidget, StatsTableWidget, TimeTableWidget};

mod template;
pub use template::*;
