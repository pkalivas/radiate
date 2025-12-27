mod app;
mod chart;
mod runtime;
mod state;
mod styles;
mod widgets;

use crate::runtime::UiRuntime;
use radiate_engines::{Chromosome, GeneticEngine};
use std::time::Duration;

pub const DEFAULT_RENDER_INTERVAL: Duration = Duration::from_millis(100);

pub fn ui<C, T>(engine: impl Into<UiInput<C, T>>) -> UiRuntime<C, T>
where
    C: Chromosome + Clone + 'static,
    T: Clone + Send + Sync,
{
    let (engine, render_interval) = match engine.into() {
        UiInput::Engine(e) => (e, DEFAULT_RENDER_INTERVAL),
        UiInput::EngineRenderInterval(e, d) => (e, d),
    };

    UiRuntime::new(engine, render_interval)
}

pub enum UiInput<C, T>
where
    C: Chromosome,
    T: Clone + Send + Sync + 'static,
{
    Engine(GeneticEngine<C, T>),
    EngineRenderInterval(GeneticEngine<C, T>, Duration),
}

impl<C, T> From<GeneticEngine<C, T>> for UiInput<C, T>
where
    C: Chromosome,
    T: Clone + Send + Sync,
{
    fn from(engine: GeneticEngine<C, T>) -> Self {
        UiInput::Engine(engine)
    }
}

impl<C, T> From<(GeneticEngine<C, T>, Duration)> for UiInput<C, T>
where
    C: Chromosome,
    T: Clone + Send + Sync,
{
    fn from(input: (GeneticEngine<C, T>, Duration)) -> Self {
        UiInput::EngineRenderInterval(input.0, input.1)
    }
}
