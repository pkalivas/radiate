use crate::{Envelope, actors::actor::ActorRef};
use std::any::Any;

pub trait AnySubscriber: Send + Sync {
    fn type_name(&self) -> &'static str;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn dispatch(&self, envelope: &dyn Any);
}

pub(crate) struct SubscriberGroup<M: Send + Sync> {
    pub(crate) handles: Vec<ActorRef<Envelope<M>>>,
}

impl<M: Send + Sync + 'static> AnySubscriber for SubscriberGroup<M> {
    fn type_name(&self) -> &'static str {
        std::any::type_name::<M>()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn dispatch(&self, envelope: &dyn Any) {
        let envelope = envelope
            .downcast_ref::<Envelope<M>>()
            .expect("incorrect envelope type");

        for handle in &self.handles {
            handle.tell(envelope.clone());
        }
    }
}
