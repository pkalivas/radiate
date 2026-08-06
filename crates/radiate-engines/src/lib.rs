mod actions;
pub mod builder;
pub mod context;
pub mod engine;
mod generation;
mod io;
mod limit;
pub mod message;
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
pub use generation::{Generation, GenerationView};
pub use io::{FileReader, FileWriter, JsonReader, JsonWriter};
pub use limit::Limit;
pub use message::{
    EngineMessage, EngineStop, EpochComplete, EventCtx, EventHandler, EventId, Improvement,
    LimitTriggered, LogLevel, LoggingHandler,
};
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
use tracing_subscriber::EnvFilter;

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

    let log_level = radiate_core::env_vars::log_level();

    match log_level.as_deref() {
        Some(level) => {
            let filter = EnvFilter::try_new(level).unwrap_or_else(|e| {
                eprintln!(
                    "RADIATE_LOG_LEVEL={level:?} is not a valid filter directive ({e}), falling back to \"info\""
                );
                EnvFilter::new("info")
            });

            match radiate_core::env_vars::log_format()
                .as_deref()
                .unwrap_or("compact")
            {
                "json" => init_json_logging(filter),
                "pretty" => init_pretty_logging(filter),
                _ => init_compact_logging(filter),
            }
        }
        None => disable_logging(),
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

fn init_pretty_logging(filter: EnvFilter) {
    INIT_LOGGING.lock().unwrap().call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .with_level(true)
            .pretty()
            .init();
    });
}
