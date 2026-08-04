use crate::Actor;
use crate::ActorContext;
use crate::MessageHandler;

use super::actor::ActorId;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct DeadLetter {
    pub message_type: &'static str,
    pub actor_id: ActorId,
}

pub struct DeadLetterActor {
    max_size: usize,
    queue: Arc<Mutex<Vec<DeadLetter>>>,
}

impl DeadLetterActor {
    pub fn new(max_size: usize) -> Self {
        DeadLetterActor {
            max_size,
            queue: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Actor for DeadLetterActor {}

impl MessageHandler<DeadLetter> for DeadLetterActor {
    fn handle(&mut self, message: DeadLetter, _ctx: &ActorContext) {
        let mut queue = self.queue.lock().unwrap();
        if queue.len() < self.max_size {
            queue.push(message);
        } else {
            // If the queue is full, we can choose to drop the message or handle it differently.
            // For now, we'll just drop it.
        }
    }
}

/// Fired by [`MessageBroker::subscribe`] every time a new actor is
/// registered — a lifecycle fact about the broker itself, independent of
/// whatever domain-specific messages (`EngineStart`, `Log`, ...) are
/// actually flowing through it. `subscriber_count` is the number of actors
/// now registered for `message_type`, including the one that just joined.
///
/// Subscribing to `ActorSubscribed` itself is not a special case: it goes
/// through the same `subscribe()` path as anything else, so the very call
/// that registers your `ActorSubscribed` listener immediately fires one
/// event describing that registration.
#[derive(Clone, Debug)]
pub struct ActorSubscribed {
    pub message_type: &'static str,
    pub actor_id: ActorId,
    pub subscriber_count: usize,
}

/// Fired when an actor's handler panics while processing a message. See
/// [`super::actor::Actor::drain`] for why the panic is caught in place
/// (before the `MutexGuard` around the handler would be dropped mid-unwind)
/// rather than being left to poison that actor's `Mutex` and silently kill
/// it for the rest of the process — the actor keeps handling later
/// messages after this fires. `panic_message` is best-effort: only
/// `&str`/`String` panic payloads become readable text, anything else
/// becomes a generic message.
///
/// Not re-emitted for a panic that happens while handling an
/// `ActorPanicked` itself — without that cutoff, a subscriber whose own
/// `ActorPanicked` handler always panics would flood the bus with an
/// unbounded chain of `ActorPanicked`-about-`ActorPanicked` events.
#[derive(Clone, Debug)]
pub struct ActorPanicked {
    pub actor_id: ActorId,
    pub reason: String,
}
