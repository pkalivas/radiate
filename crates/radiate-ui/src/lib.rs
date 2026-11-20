mod app;
mod chart;
mod runtime;
mod state;
mod styles;
mod widgets;

pub use crate::runtime::Tui;
use radiate_engines::{Chromosome, Engine, EngineRuntime, GeneticEngine, LogEvent};
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

pub fn ui<C, T>(engine: impl Into<UiInput<C, T>>) -> GeneticEngine<C, T>
where
    C: Chromosome + Debug + Clone + 'static,
    T: Clone + Send + Sync,
{
    let (engine, _) = match engine.into() {
        UiInput::Engine(e) => (e, DEFAULT_RENDER_INTERVAL),
        UiInput::EngineRenderInterval(e, d) => (e, d),
    };

    let context = engine.context();
    let control = context.sync();
    let stream = context.event_stream();

    let logging_handlers = stream.get_subsribers::<LogEvent>();
    for handler in &logging_handlers {
        stream.unsubscribe(handler.id);
    }

    let handle = Tui::<C, T>::new(control);

    stream.attach(handle).unwrap();

    engine
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

// pub fn ui<C, T>(engine: impl Into<UiInput<C, T>>) -> TuiEngine<C, T>
// where
//     C: Chromosome + Debug + Clone + 'static,
//     T: Clone + Send + Sync,
// {
//     disable_logging();
//     let (engine, render_interval) = match engine.into() {
//         UiInput::Engine(e) => (e, DEFAULT_RENDER_INTERVAL),
//         UiInput::EngineRenderInterval(e, d) => (e, d),
//     };

//     TuiEngine::new(engine, render_interval)
// }

// pub enum UiInput<C, T>
// where
//     C: Chromosome,
//     T: Clone + Send + Sync + 'static,
// {
//     Engine(GeneticEngine<C, T>),
//     EngineRenderInterval(GeneticEngine<C, T>, Duration),
// }

// impl<C, T> From<GeneticEngine<C, T>> for UiInput<C, T>
// where
//     C: Chromosome,
//     T: Clone + Send + Sync,
// {
//     fn from(engine: GeneticEngine<C, T>) -> Self {
//         UiInput::Engine(engine)
//     }
// }

// impl<C, T> From<(GeneticEngine<C, T>, Duration)> for UiInput<C, T>
// where
//     C: Chromosome,
//     T: Clone + Send + Sync,
// {
//     fn from(input: (GeneticEngine<C, T>, Duration)) -> Self {
//         UiInput::EngineRenderInterval(input.0, input.1)
//     }
// }
