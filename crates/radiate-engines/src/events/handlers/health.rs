use crate::events::EpochComplete;
use crate::{
    Actor,
    events::{MessageHandler, Warning, addr::ActorContext},
};
use radiate_core::MetricSet;
use std::marker::PhantomData;

const NAME: &str = "HealthMonitor";
const STAGNATION_WARNING_THRESHOLD: usize = 100;
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

impl<T> Actor for HealthMonitor<T>
where
    T: Send + Sync + 'static,
{
    fn name(&self) -> &str {
        NAME
    }

    fn started(&mut self, ctx: &ActorContext<Self>)
    where
        Self: Sized,
    {
        ctx.subscribe::<EpochComplete<T>>();
    }
}

impl<T> MessageHandler<EpochComplete<T>> for HealthMonitor<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, message: &EpochComplete<T>, ctx: &ActorContext<Self>) {
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
