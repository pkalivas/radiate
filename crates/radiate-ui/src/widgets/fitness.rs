use crate::{
    state::AppState,
    widgets::{ChartWidget, ParetoPagingWidget},
};
use radiate_engines::Chromosome;
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
            let charts = if state.display_mini_chart_mean() {
                vec![
                    chart_state.fitness_chart(),
                    chart_state.fitness_mean_chart(),
                ]
            } else {
                vec![chart_state.fitness_chart()]
            };

            ChartWidget::from(charts).render(area, buf);
        } else {
            ParetoPagingWidget::new(&state).render(area, buf);
        }
    }
}
