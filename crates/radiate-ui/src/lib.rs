mod app;
mod chart;
mod keys;
mod state;
mod styles;
mod tui;
mod widgets;

pub use crate::tui::Tui;
use radiate_engines::{Chromosome, EngineRuntime, EpochComplete, Expr, GeneticEngine};
use std::fmt::Debug;

pub trait TuiRuntimeExt {
    fn ui(self) -> Self;
    fn pause_every(self, interval: usize) -> Self;
}

impl<C, T> TuiRuntimeExt for EngineRuntime<GeneticEngine<C, T>>
where
    C: Chromosome + Debug + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    fn ui(self) -> Self {
        let context = self.context();

        let handle = Tui::<C, T>::new(context.sync());

        self.attach(handle).logging()
    }

    fn pause_every(self, interval: usize) -> Self {
        assert!(
            interval > 0,
            "pause_every interval must be greater than zero"
        );
        let guarded_interval = interval.max(1);
        let context = self.context();
        let control = context.sync();
        let stream = context.stream();

        if !stream.handler_registered::<Tui<C, T>>() {
            panic!("Cannot call 'pause_every' before attaching the Tui handler");
        }

        stream
            .subscribe(move |_: &EpochComplete<T>| {
                control.set_paused(true);
            })
            .schedule(Expr::every(guarded_interval))
            .unwrap();

        self
    }
}
