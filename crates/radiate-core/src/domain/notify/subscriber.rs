use crate::{
    Envelope, Executor, Message,
    notify::{
        actor::Actor,
        broker::{ActorMeta, SubscriptionMeta},
        message::EventContext,
    },
};
use std::{
    any::Any,
    fmt::{self, Debug},
    sync::Arc,
};

pub trait AnySubscription: Send + Sync + fmt::Debug {
    fn type_name(&self) -> &'static str;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn send(&self, envelope: &dyn Any, ctx: EventContext, executor: &Executor);

    fn num_actors(&self) -> usize;

    fn queued(&self) -> usize;

    fn processed(&self) -> u64;
    fn meta(&self) -> SubscriptionMeta;
}

#[derive(Debug)]
pub(super) struct Subscription<M: Message> {
    pub(super) actors: Vec<Arc<Actor<M>>>,
}

impl<M: Message + Debug> AnySubscription for Subscription<M> {
    fn type_name(&self) -> &'static str {
        std::any::type_name::<M>()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    #[inline]
    fn send(&self, envelope: &dyn Any, ctx: EventContext, executor: &Executor) {
        let envelope = envelope
            .downcast_ref::<Envelope<M>>()
            .expect("incorrect envelope type");

        for actor in &self.actors {
            actor.tell(envelope.clone(), ctx.clone(), executor);
        }
    }

    fn num_actors(&self) -> usize {
        self.actors.len()
    }

    fn queued(&self) -> usize {
        self.actors.iter().map(|a| a.mailbox_len()).sum()
    }

    fn processed(&self) -> u64 {
        self.actors.iter().map(|a| a.num_processed()).sum()
    }

    fn meta(&self) -> SubscriptionMeta {
        let actors = self
            .actors
            .iter()
            .map(|actor| ActorMeta {
                id: actor.id(),
                queued: actor.mailbox_len(),
                processed: actor.num_processed(),
            })
            .collect();

        SubscriptionMeta {
            type_name: self.type_name(),
            actors,
        }
    }
}
