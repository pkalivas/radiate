mod actions;
pub mod builder;
pub mod context;
pub mod engine;
mod events;
mod generation;
mod io;
mod limit;
mod pipeline;
pub mod runtime;
mod steps;

use std::sync::atomic::{AtomicBool, Ordering};

pub use builder::GeneticEngineBuilder;
pub use context::EvolutionContext;
pub use engine::GeneticEngine;
pub use events::{
    EngineEvent, EngineMessage, EpochCompleted, EpochCompletedData, EpochStarted, EpochStartedData,
    EventBus, Improved, ImprovedData, LoggingHandler, MetricCollector, Started, StartedData,
    Stopped, StoppedData,
};
pub use generation::{Generation, GenerationView};
pub use io::{FileReader, FileWriter, JsonReader, JsonWriter};
pub use limit::Limit;
pub use runtime::EngineRuntime;

pub use steps::{
    EngineStep, EvaluateStep, OffspringConfig, RecombineStep, SelectConfig, SpeciateStep,
    SurvivorConfig,
};

pub use radiate_alters::*;
pub use radiate_core::*;
pub use radiate_error::{RadiateError, ensure, radiate_err};
pub use radiate_selectors::*;
pub use radiate_utils::Shape;

pub(crate) type Result<T> = std::result::Result<T, RadiateError>;

pub fn init_logging() {
    pub use std::sync::Once;
    static INIT_LOGGING: Once = Once::new();
    static LOGGING_INITIALIZED: AtomicBool = AtomicBool::new(false);

    if LOGGING_INITIALIZED.load(Ordering::SeqCst) {
        return;
    }

    INIT_LOGGING.call_once(|| {
        use tracing_subscriber::fmt::format::FmtSpan;
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

        LOGGING_INITIALIZED.store(true, Ordering::SeqCst);

        std::panic::set_hook(Box::new(|info| {
            tracing::error!("PANIC: {}", info);
        }));

        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                    .with_target(false)
                    .with_level(true)
                    .compact(),
            )
            .init();
    });
}
