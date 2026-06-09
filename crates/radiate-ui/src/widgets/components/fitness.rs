use crate::{
    state::{AppState, LineChartType},
    widgets::{AppWidget, components::LineChartWidget, components::ParetoPagingWidget},
};
use radiate_engines::{Chromosome, metric_names};
use ratatui::widgets::Widget;

pub struct FitnessChartPanelWidget;

impl FitnessChartPanelWidget {
    pub fn new() -> Self {
        Self
    }
}

impl<C: Chromosome> AppWidget<C> for FitnessChartPanelWidget {
    fn render(
        &self,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
        state: &mut AppState<C>,
    ) {
        if state.evo.pareto.objective.is_single() {
            let chart_state = &state.evo.charts;
            let charts =
                vec![chart_state.get_line_chart(&metric_names::BEST_SCORES, LineChartType::Value)];

            LineChartWidget::from(charts).render(area, buf);
        } else {
            ParetoPagingWidget::new(state).render(area, buf);
        }
    }
}
