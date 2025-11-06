use super::EventHandler;
use crate::events::events::*;
use radiate_core::{Chromosome, Executor};
use std::sync::{Arc, Mutex};

type Subscriber<T> = Arc<Mutex<dyn EventHandler<T>>>;

#[derive(Clone)]
pub struct EventBus<T> {
    handlers: Vec<Subscriber<T>>,
    executor: Arc<Executor>,
}

impl<T> EventBus<T> {
    pub fn new(executor: Arc<Executor>, handlers: Vec<Subscriber<T>>) -> Self {
        EventBus { handlers, executor }
    }

    pub fn publish<'a, C>(&self, message: EngineMessage<'a, C, T>)
    where
        C: Chromosome,
        T: Clone + Send + Sync + 'static,
    {
        if self.handlers.is_empty() {
            return;
        }

        let event = match message {
            EngineMessage::Start => EngineEvent::Start,
            EngineMessage::Stop(ctx) => EngineEvent::Stop(
                ctx.best.clone(),
                ctx.metrics.clone(),
                ctx.score.clone().unwrap_or_default(),
            ),
            EngineMessage::EpochStart(ctx) => EngineEvent::EpochStart(ctx.index),
            EngineMessage::EpochEnd(ctx) => EngineEvent::EpochComplete(
                ctx.index,
                ctx.best.clone(),
                ctx.metrics.clone(),
                ctx.score.clone().unwrap_or_default(),
            ),
            EngineMessage::Improvement(ctx) => EngineEvent::Improvement(
                ctx.index,
                ctx.best.clone(),
                ctx.score.clone().unwrap_or_default(),
            ),
        };

        let wrapped_event = Arc::new(event);
        for handler in self.handlers.iter() {
            let clone_handler = Arc::clone(handler);
            let clone_event = Arc::clone(&wrapped_event);
            self.executor.submit(move || {
                clone_handler.lock().unwrap().handle(&clone_event);
            });
        }
    }
}
