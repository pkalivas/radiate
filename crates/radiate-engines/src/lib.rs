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
    Mutex, Once,
    atomic::{AtomicBool, Ordering},
};
use tracing_subscriber::EnvFilter;

pub use builder::GeneticEngineBuilder;
pub use context::EvolutionContext;
pub use engine::GeneticEngine;
pub use events::*;
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

pub(crate) type Result<T> = std::result::Result<T, RadiateError>;

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

    let filter = EnvFilter::new(radiate_core::env_vars::DEFAULT_LOG_LEVEL);
    match radiate_core::env_vars::log_format()
        .as_deref()
        .unwrap_or(radiate_core::env_vars::COMPACT_LOGGING)
    {
        radiate_core::env_vars::JSON_LOGGING => init_json_logging(filter),
        _ => init_compact_logging(filter),
    }

    LOGGING_INITIALIZED.store(true, Ordering::SeqCst);

    std::panic::set_hook(Box::new(|info| {
        tracing::error!("{}", info);
    }));
}

fn init_compact_logging(filter: EnvFilter) {
    INIT_LOGGING.lock().unwrap().call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .with_level(true)
            .compact()
            .init();
    });
}

fn init_json_logging(filter: EnvFilter) {
    INIT_LOGGING.lock().unwrap().call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .with_level(true)
            .json()
            .init();
    });
}
