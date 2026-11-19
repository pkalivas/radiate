mod app;
mod chart;
mod runtime;
mod state;
mod styles;
mod widgets;

pub use crate::runtime::TuiHandle;
use radiate_engines::{Chromosome, Engine, EngineRuntime, GeneticEngine, disable_logging};
use std::fmt::Debug;
use std::time::Duration;

pub const DEFAULT_RENDER_INTERVAL: Duration = Duration::from_millis(100);

// radiate-ui
pub trait TuiRuntimeExt {
    fn ui(self) -> Self;
}

impl<C, T> TuiRuntimeExt for EngineRuntime<GeneticEngine<C, T>>
where
    C: Chromosome + Debug + Clone + 'static,
    T: Clone + Send + Sync + 'static,
{
    fn ui(self) -> Self {
        let control = self.control();
        self.attach(TuiHandle::<C, T>::new(control))
    }
}

pub fn ui<C, T>(engine: impl Into<UiInput<C, T>>) -> GeneticEngine<C, T>
where
    C: Chromosome + Debug + Clone + 'static,
    T: Clone + Send + Sync,
{
    let (engine, render_interval) = match engine.into() {
        UiInput::Engine(e) => (e, DEFAULT_RENDER_INTERVAL),
        UiInput::EngineRenderInterval(e, d) => (e, d),
    };

    let context = engine.context();
    let control = context.get_or_create_sync();
    let handle = TuiHandle::<C, T>::new(control);
    context.event_stream().attach(handle).unwrap();

    engine

    // // disable_logging();
    // let (engine, render_interval) = match engine.into() {
    //     UiInput::Engine(e) => (e, DEFAULT_RENDER_INTERVAL),
    //     UiInput::EngineRenderInterval(e, d) => (e, d),
    // };

    // engine
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
