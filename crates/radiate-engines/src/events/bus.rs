use super::EventHandler;
use crate::{
    EvolutionContext,
    events::{
        handlers::{
            OnEpochComplete, OnEpochCompleteAdapter, OnEpochStart, OnEpochStartAdapter,
            OnImprovement, OnImprovementAdapter, OnStart, OnStartAdapter, OnStop, OnStopAdapter,
        },
        message::{EngineEvent, EngineEventInner, EngineMessage, EventType},
    },
};
use radiate_core::{Chromosome, Executor};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

type Subscriber<T> = Arc<Mutex<dyn EventHandler<T>>>;

#[derive(Clone)]
pub struct EventBus<T> {
    handlers: HashMap<EventType, Vec<Subscriber<T>>>,
    executor: Arc<Executor>,
}

impl<T> EventBus<T> {
    pub fn new(executor: Arc<Executor>, handlers: HashMap<EventType, Vec<Subscriber<T>>>) -> Self {
        EventBus { handlers, executor }
    }

    pub fn handlers(&self) -> HashMap<EventType, Vec<Subscriber<T>>> {
        self.handlers.clone()
    }

    pub fn subscribe<H>(&mut self, handler: H)
    where
        H: EventHandler<T> + 'static,
    {
        self.handlers
            .entry(EventType::All)
            .or_default()
            .push(Arc::new(Mutex::new(handler)));
    }

    pub fn subscribe_typed<H>(&mut self, event_type: EventType, handler: H)
    where
        H: EventHandler<T> + 'static,
    {
        self.handlers
            .entry(event_type)
            .or_default()
            .push(Arc::new(Mutex::new(handler)));
    }

    pub fn on_start<H>(&mut self, handler: H)
    where
        H: OnStart + 'static,
        T: 'static,
    {
        self.subscribe_typed(EventType::Start, OnStartAdapter(handler));
    }

    pub fn on_stop<H>(&mut self, handler: H)
    where
        H: OnStop<T> + 'static,
        T: 'static,
    {
        self.subscribe_typed(EventType::Stop, OnStopAdapter(handler));
    }

    pub fn on_epoch_start<H>(&mut self, handler: H)
    where
        H: OnEpochStart<T> + 'static,
        T: 'static,
    {
        self.subscribe_typed(EventType::EpochStart, OnEpochStartAdapter(handler));
    }

    pub fn on_epoch_complete<H>(&mut self, handler: H)
    where
        H: OnEpochComplete<T> + 'static,
        T: 'static,
    {
        self.subscribe_typed(EventType::EpochComplete, OnEpochCompleteAdapter(handler));
    }

    pub fn on_improvement<H>(&mut self, handler: H)
    where
        H: OnImprovement<T> + 'static,
        T: 'static,
    {
        self.subscribe_typed(EventType::Improvement, OnImprovementAdapter(handler));
    }

    pub fn publish<C>(&self, message: EngineMessage<C, T>)
    where
        C: Chromosome,
        T: Clone + Send + Sync + 'static,
    {
        if self.handlers.is_empty() {
            return;
        }

        let event_type = message.event_type();
        let specific = self
            .handlers
            .get(&event_type)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let wildcard = self
            .handlers
            .get(&EventType::All)
            .map(Vec::as_slice)
            .unwrap_or(&[]);

        if specific.is_empty() && wildcard.is_empty() {
            return;
        }

        let event = match message {
            EngineMessage::Start(ctx) => to_start_event(ctx),
            EngineMessage::Stop(ctx) => to_stop_event(ctx),
            EngineMessage::EpochStart(ctx) => to_epoch_start_event(ctx),
            EngineMessage::EpochEnd(ctx) => to_epoch_complete_event(ctx),
            EngineMessage::Improvement(ctx) => to_improvement_event(ctx),
        };

        for handler in specific.iter().chain(wildcard.iter()) {
            let clone_handler = Arc::clone(handler);
            let clone_event = event.clone();
            self.executor.submit(move || {
                clone_handler.lock().unwrap().handle(clone_event);
            });
        }
    }
}

fn to_improvement_event<C, T>(ctx: &mut EvolutionContext<C, T>) -> EngineEvent<T>
where
    C: Chromosome,
    T: Clone,
{
    let sync = ctx.get_or_create_control();
    EngineEvent::new(
        sync,
        EngineEventInner::Improvement(
            ctx.index,
            ctx.best.clone(),
            ctx.score.clone().unwrap_or_default(),
        ),
    )
}

fn to_epoch_complete_event<C, T>(ctx: &mut EvolutionContext<C, T>) -> EngineEvent<T>
where
    C: Chromosome,
    T: Clone,
{
    EngineEvent::new(
        ctx.get_or_create_control(),
        EngineEventInner::EpochComplete(
            ctx.index,
            ctx.best.clone(),
            ctx.metrics.clone(),
            ctx.score.clone().unwrap_or_default(),
            ctx.objective.clone(),
        ),
    )
}

fn to_epoch_start_event<C, T>(ctx: &mut EvolutionContext<C, T>) -> EngineEvent<T>
where
    C: Chromosome,
    T: Clone,
{
    let sync = ctx.get_or_create_control();
    EngineEvent::new(sync, EngineEventInner::EpochStart(ctx.index))
}

fn to_stop_event<C, T>(ctx: &mut EvolutionContext<C, T>) -> EngineEvent<T>
where
    C: Chromosome,
    T: Clone,
{
    EngineEvent::new(
        ctx.get_or_create_control(),
        EngineEventInner::Stop(
            ctx.index,
            ctx.best.clone(),
            ctx.metrics.clone(),
            ctx.score.clone().unwrap_or_default(),
        ),
    )
}

fn to_start_event<C, T>(ctx: &mut EvolutionContext<C, T>) -> EngineEvent<T>
where
    C: Chromosome,
    T: Clone,
{
    let sync = ctx.get_or_create_control();
    EngineEvent::new(sync, EngineEventInner::Start)
}

impl<T> Default for EventBus<T> {
    fn default() -> Self {
        EventBus {
            handlers: HashMap::new(),
            executor: Arc::new(Executor::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radiate_core::BitChromosome;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Clone, Default)]
    struct Counter(Arc<AtomicUsize>);

    impl Counter {
        fn count(&self) -> usize {
            self.0.load(Ordering::SeqCst)
        }
    }

    impl OnStart for Counter {
        fn on_start(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    // #[test]
    // fn on_start_only_fires_for_start_events() {
    //     let mut bus: EventBus<i32> = EventBus::new(Arc::new(Executor::default()), HashMap::new());

    //     let start_only = Counter::default();
    //     bus.on_start(start_only.clone());

    //     bus.publish::<BitChromosome>(EngineMessage::Start(&mut EvolutionContext::default()));

    //     assert_eq!(start_only.count(), 1);
    // }

    // #[test]
    // fn typed_subscriber_does_not_receive_unrelated_event_kinds() {
    //     let mut bus: EventBus<i32> = EventBus::new(Arc::new(Executor::default()), HashMap::new());

    //     // Subscribed only to Start; publishing a Start event should not touch
    //     // any bucket other than `EventType::Start` / `EventType::All`.
    //     let start_only = Counter::default();
    //     bus.on_start(start_only.clone());

    //     assert!(!bus.handlers.contains_key(&EventType::EpochStart));
    //     assert!(!bus.handlers.contains_key(&EventType::Improvement));

    //     bus.publish::<BitChromosome>(EngineMessage::Start(&mut EvolutionContext::default()));
    //     assert_eq!(start_only.count(), 1);
    // }

    // #[test]
    // fn wildcard_subscriber_receives_events_regardless_of_kind() {
    //     let mut bus: EventBus<i32> = EventBus::new(Arc::new(Executor::default()), HashMap::new());

    //     let hits = Arc::new(AtomicUsize::new(0));
    //     let hits_clone = Arc::clone(&hits);
    //     bus.subscribe(move |_event: &EngineEvent<i32>| {
    //         hits_clone.fetch_add(1, Ordering::SeqCst);
    //     });

    //     bus.publish::<BitChromosome>(EngineMessage::Start(&mut EvolutionContext::default()));

    //     assert_eq!(hits.load(Ordering::SeqCst), 1);
    // }

    // #[test]
    // fn publish_with_no_subscribers_does_not_panic() {
    //     let bus: EventBus<i32> = EventBus::new(Arc::new(Executor::default()), HashMap::new());
    //     bus.publish::<BitChromosome>(EngineMessage::Start(&mut EvolutionContext::default()));
    // }
}
