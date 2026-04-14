mod dashboard;
mod fitness;
mod metric;
mod status;
mod tags;

pub use dashboard::EngineDashboardPanelWidget;
pub use fitness::FitnessChartPanelWidget;
pub use metric::MetricDetailPanelWidget;
pub use status::{EngineStatusPanelWidget, MetricModalWidget};
pub use tags::TagsPanelWidget;
