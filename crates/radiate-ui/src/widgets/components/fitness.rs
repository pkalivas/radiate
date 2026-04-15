use crate::{
    state::{AppState, LineChartType},
    widgets::{components::LineChartWidget, components::ParetoPagingWidget},
};
use radiate_engines::{Chromosome, metric_names};
use ratatui::widgets::{StatefulWidget, Widget};

pub struct FitnessChartPanelWidget<C: Chromosome> {
    _phantom: std::marker::PhantomData<C>,
}

impl<C: Chromosome> FitnessChartPanelWidget<C> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<C: Chromosome> StatefulWidget for FitnessChartPanelWidget<C> {
    type State = AppState<C>;

    fn render(
        self,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
        state: &mut Self::State,
    ) {
        if state.objective_state.objective.is_single() {
            let chart_state = state.chart_state();
            // let charts = if state.display_mini_chart() {
            //     vec![
            //         chart_state.get_by_key(metric_names::BEST_SCORES, ChartType::Value),
            //         chart_state.get_by_key(metric_names::BEST_SCORES, ChartType::Mean),
            //     ]
            // } else {
            // };
            let charts =
                vec![chart_state.get_line_chart(metric_names::BEST_SCORES, LineChartType::Value)];

            LineChartWidget::from(charts).render(area, buf);
        } else {
            ParetoPagingWidget::new(&state).render(area, buf);
        }
    }
}
