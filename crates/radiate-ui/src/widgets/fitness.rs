use crate::{
    state::{AppState, ChartType},
    widgets::{ChartWidget, ParetoPagingWidget},
};
use radiate_engines::{Chromosome, metric_names};
use ratatui::widgets::{StatefulWidget, Widget};

pub struct FitnessWidget<C: Chromosome> {
    _phantom: std::marker::PhantomData<C>,
}

impl<C: Chromosome> FitnessWidget<C> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<C: Chromosome> StatefulWidget for FitnessWidget<C> {
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
            let charts = vec![chart_state.get_by_key(metric_names::BEST_SCORES, ChartType::Value)];

            ChartWidget::from(charts).render(area, buf);
        } else {
            ParetoPagingWidget::new(&state).render(area, buf);
        }
    }
}
