use crate::events::EpochComplete;
use crate::{
    Handler,
    events::{EventContext, EventHandler, Warning},
};
use radiate_core::Objective;
use std::marker::PhantomData;

const STAGNATION_WARNING_THRESHOLD: usize = 200;
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

impl<T> EventHandler for HealthMonitor<T>
where
    T: Send + Sync + 'static,
{
    fn start(&mut self, ctx: &EventContext<'_, Self>) {
        ctx.subscribe::<EpochComplete<T>>();
    }
}

impl<T> Handler<EpochComplete<T>> for HealthMonitor<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, message: &EpochComplete<T>, ctx: &EventContext<'_, Self>) {
        check_stagnation(message, ctx);
        check_diversity(message, ctx);
        check_species_collapse(message, ctx);
    }
}

fn check_stagnation<T, H>(message: &EpochComplete<T>, ctx: &EventContext<'_, H>) {
    if let Objective::Multi(_) = message.objective { return }

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

fn check_diversity<T, H>(message: &EpochComplete<T>, ctx: &EventContext<'_, H>) {
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

fn check_species_collapse<T, H>(message: &EpochComplete<T>, ctx: &EventContext<'_, H>) {
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
