mod app;
mod chart;
mod runtime;
mod state;
mod styles;
mod widgets;

pub use crate::runtime::Tui;
use radiate_engines::{Chromosome, EngineRuntime, GeneticEngine};
use std::fmt::Debug;
use std::time::Duration;

pub const DEFAULT_RENDER_INTERVAL: Duration = Duration::from_millis(100);

pub trait TuiRuntimeExt {
    fn ui(self) -> Self;
}

impl<C, T> TuiRuntimeExt for EngineRuntime<GeneticEngine<C, T>>
where
    C: Chromosome + Debug + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    fn ui(self) -> Self {
        let context = self.context();
        let control = context.sync();

        let handle = Tui::<C, T>::new(control);

        self.attach(handle).logging()
    }
}
