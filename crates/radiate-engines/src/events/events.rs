use std::{
    fmt::{Debug, Display},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use radiate_core::{Chromosome, MetricSet, Score, engine::Context};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct EventId(usize);

impl EventId {
    pub fn next() -> Self {
        static EVENT_ID: AtomicUsize = AtomicUsize::new(0);
        EventId(EVENT_ID.fetch_add(1, Ordering::SeqCst))
    }
}

pub struct Event<T> {
    id: EventId,
    data: Arc<T>,
}

impl<T> Event<T> {
    pub fn new(data: T) -> Self {
        Event {
            id: EventId::next(),
            data: Arc::new(data),
        }
    }

    pub fn id(&self) -> EventId {
        self.id
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn event_type(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}

impl<T> Clone for Event<T> {
    fn clone(&self) -> Self {
        Event {
            id: self.id,
            data: Arc::clone(&self.data),
        }
    }
}

impl<T: Debug> Debug for Event<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Event")
            .field("event_id", &self.id)
            .field("data", &self.data)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub enum EngineEvent<T> {
    Start,
    Stop,
    EpochStart(usize),
    EpochComplete {
        index: usize,
        metrics: MetricSet,
        best: T,
        score: Score,
    },
    StepStart(&'static str),
    StepComplete(&'static str),
}

impl<T> EngineEvent<T> {
    pub fn start() -> Self {
        EngineEvent::Start
    }

    pub fn stop<C>(context: &Context<C, T>) -> Self
    where
        C: Chromosome,
    {
        EngineEvent::Stop
    }

    pub fn epoch_start<C>(context: &Context<C, T>) -> Self
    where
        C: Chromosome,
    {
        EngineEvent::EpochStart(context.index)
    }

    pub fn epoch_complete<C>(context: &Context<C, T>) -> Self
    where
        C: Chromosome,
    {
        EngineEvent::EpochComplete {
            index: context.index,
            metrics: context.metrics.clone(),
            best: context
                .problem
                .decode(context.ecosystem.population().get(0).unwrap().genotype()),
            score: context.score.clone().unwrap_or_default(),
        }
    }

    pub fn step_start(step: &'static str) -> Self {
        EngineEvent::StepStart(step)
    }

    pub fn step_complete(step: &'static str) -> Self {
        EngineEvent::StepComplete(step)
    }
}

impl<T> Display for EngineEvent<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineEvent::Start => write!(f, "EngineEvent::Started"),
            EngineEvent::Stop => write!(f, "EngineEvent::Stopped"),
            EngineEvent::EpochStart(index) => write!(f, "EngineEvent::EpochStarted [{}]", index),
            EngineEvent::EpochComplete { index, .. } => {
                write!(f, "EngineEvent::EpochComplete [{}]", index)
            }
            EngineEvent::StepStart(step) => write!(f, "EngineEvent::StepStarted [{}]", step),
            EngineEvent::StepComplete(step) => write!(f, "EngineEvent::StepCompleted [{}]", step),
        }
    }
}
