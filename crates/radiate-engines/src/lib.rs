mod actions;
pub mod builder;
pub mod context;
pub mod engine;
pub mod events;
mod generation;
mod io;
mod limit;
mod pipeline;
pub mod runtime;
mod steps;

use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};

pub use builder::GeneticEngineBuilder;
pub use context::EvolutionContext;
pub use engine::GeneticEngine;
pub use events::{EpochComplete, LimitTriggered, LoggingHandler, MetricCollector};
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

pub use std::sync::Once;
static INIT_LOGGING: Mutex<Once> = Mutex::new(Once::new());
static LOGGING_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub fn disable_logging() {
    LOGGING_INITIALIZED.store(true, Ordering::SeqCst);
    INIT_LOGGING.lock().unwrap().call_once(|| {
        tracing::subscriber::set_global_default(tracing::subscriber::NoSubscriber::default())
            .expect("Failed to set global subscriber");
    });
}

pub fn init_logging() {
    if LOGGING_INITIALIZED.load(Ordering::SeqCst) {
        return;
    }

    INIT_LOGGING.lock().unwrap().call_once(|| {
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
