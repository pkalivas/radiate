use crate::{
    EngineStop, EventId,
    message::{CheckpointSaved, Warning},
};
use crate::{EventHandler, message::stream::EventCtx};
use crate::{LimitTriggered, message::EpochComplete};
use radiate_core::{EngineState, Objective};

const STAGNATION_WARNING_THRESHOLD: usize = 5;
const DIVERSITY_WARNING_THRESHOLD: f32 = 0.1;
const LARGEST_SPECIES_SHARE_WARNING_THRESHOLD: f32 = 0.9;

#[derive(Clone)]
pub struct HealthMonitorHandler<T>(std::marker::PhantomData<T>);

impl<T> EventHandler<EpochComplete<T>> for HealthMonitorHandler<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, message: &EpochComplete<T>, ctx: &EventCtx) {
        check_stagnation(message, ctx);
        check_diversity(message, ctx);
        check_species_collapse(message, ctx);
    }
}

impl<T> Default for HealthMonitorHandler<T> {
    fn default() -> Self {
        HealthMonitorHandler(std::marker::PhantomData)
    }
}

fn check_stagnation<T>(message: &EpochComplete<T>, ctx: &EventCtx) {
    let stag_count = message
        .metrics
        .stagnation_count()
        .map(|met| met.last_value() as usize)
        .unwrap_or_default();

    if stag_count >= STAGNATION_WARNING_THRESHOLD {
        ctx.publish(Warning(format!(
            "Stagnation detected: {} epochs without improvement",
            stag_count
        )));
    }
}

fn check_diversity<T>(message: &EpochComplete<T>, ctx: &EventCtx) {
    let Some(ratio) = message.metrics.diversity_ratio() else {
        return;
    };

    let ratio = ratio.last_value();
    if ratio < DIVERSITY_WARNING_THRESHOLD {
        ctx.publish(Warning(format!(
            "Diversity collapse: only {:.1}% of the population is genetically distinct",
            ratio * 100.0
        )));
    }
}

fn check_species_collapse<T>(message: &EpochComplete<T>, ctx: &EventCtx) {
    let Some(share) = message.metrics.largest_species_share() else {
        return;
    };

    let share = share.last_value();
    if share >= LARGEST_SPECIES_SHARE_WARNING_THRESHOLD {
        ctx.publish(Warning(format!(
            "Species collapse: the largest species holds {:.1}% of the population",
            share * 100.0
        )));
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogLevel {
    Info,
    Warn,
}

#[derive(Clone, Debug)]
pub struct LogEvent(pub LogLevel, pub String);

#[derive(Clone, Default)]
pub struct LoggingHandler;

impl EventHandler<LogEvent> for LoggingHandler {
    fn handle(&mut self, message: &LogEvent, ctx: &EventCtx) {
        log_event(ctx.id(), message.0, message.1.clone());
    }
}

impl EventHandler<LimitTriggered> for LoggingHandler {
    fn handle(&mut self, message: &LimitTriggered, ctx: &EventCtx) {
        ctx.publish(LogEvent(
            LogLevel::Info,
            format!("Limit triggered: {:?}", message.1),
        ));
    }
}

impl EventHandler<Warning> for LoggingHandler {
    fn handle(&mut self, message: &Warning, ctx: &EventCtx) {
        ctx.publish(LogEvent(LogLevel::Warn, message.0.clone()));
    }
}

impl EventHandler<CheckpointSaved> for LoggingHandler {
    fn handle(&mut self, message: &CheckpointSaved, ctx: &EventCtx) {
        ctx.publish(LogEvent(
            LogLevel::Info,
            format!(
                "Checkpoint saved at index {}: {}",
                message.index, message.path
            ),
        ));
    }
}

impl<T> EventHandler<EpochComplete<T>> for LoggingHandler
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, event: &EpochComplete<T>, ctx: &EventCtx) {
        let time = event
            .metrics
            .time()
            .and_then(|m| m.times().map(|t| t.sum()))
            .unwrap_or_default();

        match event.objective {
            Objective::Single(_) => {
                ctx.publish(LogEvent(
                    LogLevel::Info,
                    format!(
                        "Epoch {:<4} | Score: {:>8.4} | Time: {:>5.2?}",
                        event.index,
                        event.score.as_f32(),
                        time
                    ),
                ));
            }
            Objective::Multi(_) => {
                let front_size = event.metrics.front_size();
                let front_size_value = front_size.map(|ent| ent.last_value()).unwrap_or(0.0);

                ctx.publish(LogEvent(
                    LogLevel::Info,
                    format!(
                        "Epoch {:<4} | Front Size: {:.3} | Time: {:>5.2?}",
                        event.index, front_size_value, time
                    ),
                ));
            }
        }
    }
}

impl EventHandler<EngineState> for LoggingHandler {
    fn handle(&mut self, event: &EngineState, ctx: &EventCtx) {
        ctx.publish(LogEvent(
            LogLevel::Info,
            format!("State Change: {:?}", event),
        ));
    }
}

impl<T> EventHandler<EngineStop<T>> for LoggingHandler
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, _: &EngineStop<T>, ctx: &EventCtx) {
        ctx.publish(LogEvent(LogLevel::Info, format!("Engine stopped")));
    }
}

fn log_event(id: &EventId, level: LogLevel, message: String) {
    match level {
        LogLevel::Info => tracing::info!("{} - {}", id, message),
        LogLevel::Warn => tracing::warn!("{} - {}", id, message),
    }
}
