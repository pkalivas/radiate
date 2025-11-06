pub mod builder;
mod context;
pub mod engine;
mod events;
mod generation;
mod iter;
mod limit;
mod pipeline;
mod steps;

pub use builder::GeneticEngineBuilder;
pub use engine::GeneticEngine;
pub use events::{EngineEvent, EventBus, EventHandler};
pub use generation::Generation;
pub use iter::EngineIteratorExt;
pub use limit::Limit;
pub use steps::EvaluateStep;

pub use radiate_alters::*;
pub use radiate_core::*;
pub use radiate_error::{RadiateError, ensure, radiate_err};
pub use radiate_selectors::*;

pub(crate) type Result<T> = std::result::Result<T, RadiateError>;

pub fn init_logging() {
    pub use std::sync::Once;
    static INIT_LOGGING: Once = Once::new();

    INIT_LOGGING.call_once(|| {
        use tracing_subscriber::fmt::format::FmtSpan;
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
