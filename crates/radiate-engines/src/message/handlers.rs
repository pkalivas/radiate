use crate::{
    Actor, EngineStop, EventHandler,
    message::{CheckpointSaved, MessageHandler, Warning, actor::ActorContext},
};
use crate::{LimitTriggered, message::EpochComplete};
use radiate_core::MetricSet;
use radiate_core::{EngineState, Objective};
use std::marker::PhantomData;
use std::sync::Arc;

const STAGNATION_WARNING_THRESHOLD: usize = 5;
const DIVERSITY_WARNING_THRESHOLD: f32 = 0.1;
const LARGEST_SPECIES_SHARE_WARNING_THRESHOLD: f32 = 0.9;

pub struct HealthMonitor<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for HealthMonitor<T> {
    fn default() -> Self {
        HealthMonitor {
            _marker: PhantomData,
        }
    }
}

impl<T> Actor for HealthMonitor<T> where T: Send + Sync + 'static {}

impl<T> MessageHandler<Arc<EpochComplete<T>>> for HealthMonitor<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, message: Arc<EpochComplete<T>>, ctx: &ActorContext<Self>) {
        check_stagnation(&message.metrics, ctx);
        check_diversity(&message.metrics, ctx);
        check_species_collapse(&message.metrics, ctx);
    }
}

fn check_stagnation<A: Actor>(metrics: &MetricSet, ctx: &ActorContext<A>) {
    let stag_count = metrics
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

fn check_diversity<A: Actor>(metrics: &MetricSet, ctx: &ActorContext<A>) {
    let Some(ratio) = metrics.diversity_ratio() else {
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

fn check_species_collapse<A: Actor>(metrics: &MetricSet, ctx: &ActorContext<A>) {
    let Some(share) = metrics.largest_species_share() else {
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
    fn handle(&mut self, message: &LogEvent) {
        match message.0 {
            LogLevel::Info => tracing::info!("{}", message.1),
            LogLevel::Warn => tracing::warn!("{}", message.1),
        }
    }
}

pub struct EngineLogger<T> {
    _marker: PhantomData<T>,
}

impl<T> EngineLogger<T> {
    pub fn new() -> Self {
        EngineLogger {
            _marker: PhantomData,
        }
    }
}

impl<T> Actor for EngineLogger<T> where T: Send + Sync + 'static {}

impl<T> MessageHandler<Arc<LimitTriggered>> for EngineLogger<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, message: Arc<LimitTriggered>, ctx: &ActorContext<Self>) {
        ctx.publish(LogEvent(
            LogLevel::Info,
            format!("Limit triggered: {:?}", message.1),
        ));
    }
}

impl<T> MessageHandler<Arc<Warning>> for EngineLogger<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, message: Arc<Warning>, ctx: &ActorContext<Self>) {
        ctx.publish(LogEvent(LogLevel::Warn, message.0.clone()));
    }
}

impl<T> MessageHandler<Arc<CheckpointSaved>> for EngineLogger<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, message: Arc<CheckpointSaved>, ctx: &ActorContext<Self>) {
        ctx.publish(LogEvent(
            LogLevel::Info,
            format!(
                "Checkpoint saved at index {}: {}",
                message.index, message.path
            ),
        ));
    }
}

impl<T> MessageHandler<Arc<EpochComplete<T>>> for EngineLogger<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, event: Arc<EpochComplete<T>>, ctx: &ActorContext<Self>) {
        let time = event
            .metrics
            .time()
            .and_then(|m| m.times().map(|t| t.sum()))
            .unwrap_or_default();

        let msg = match event.objective {
            Objective::Single(_) => format!(
                "Epoch {:<4} | Score: {:>8.4} | Time: {:>5.2?}",
                event.index,
                event.score.as_f32(),
                time
            ),
            Objective::Multi(_) => {
                let front_size = event
                    .metrics
                    .front_size()
                    .map(|ent| ent.last_value())
                    .unwrap_or(0.0);
                format!(
                    "Epoch {:<4} | Front Size: {:.3} | Time: {:>5.2?}",
                    event.index, front_size, time
                )
            }
        };

        ctx.publish(LogEvent(LogLevel::Info, msg));
    }
}

impl<T> MessageHandler<Arc<EngineState>> for EngineLogger<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, event: Arc<EngineState>, ctx: &ActorContext<Self>) {
        ctx.publish(LogEvent(
            LogLevel::Info,
            format!("State Change: {:?}", *event),
        ));
    }
}

impl<T> MessageHandler<Arc<EngineStop<T>>> for EngineLogger<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, _: Arc<EngineStop<T>>, ctx: &ActorContext<Self>) {
        ctx.publish(LogEvent(LogLevel::Info, "Engine stopped".into()));
    }
}
