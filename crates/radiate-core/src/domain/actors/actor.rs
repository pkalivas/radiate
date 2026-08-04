use crate::{
    Executor,
    actors::{
        message::DeadLetter,
        system::{ActorContext, DomainBus},
    },
};
use radiate_utils::sentry_id;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::{
    any::TypeId,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};

sentry_id!(ActorId);

pub trait Actor: Send {
    type Message: Send;
    fn receive(&mut self, message: Self::Message, ctx: &ActorContext);
    fn on_child_failure(&mut self, _reason: String) {}
}

pub(crate) trait ScheduledWorker: Send + Sync {
    fn try_claim(&self) -> bool;
    fn process_batch(self: Arc<Self>);
}

pub struct ActorCell<A: Actor> {
    pub(crate) actor: Arc<Mutex<A>>,
    pub(crate) receiver: Arc<Mutex<Receiver<A::Message>>>,
    pub(crate) scheduled: AtomicBool,
    pub(crate) context: ActorContext,
}

impl<A: Actor> ScheduledWorker for ActorCell<A> {
    fn try_claim(&self) -> bool {
        self.scheduled
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    fn process_batch(self: Arc<Self>) {
        loop {
            {
                let mut actor = self.actor.lock().unwrap();
                let receiver = self.receiver.lock().unwrap();
                while let Ok(msg) = receiver.try_recv() {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        actor.receive(msg, &self.context);
                    }));

                    if let Err(payload) = result {
                        let reason = payload
                            .downcast_ref::<&str>()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "actor panicked".to_string());

                        if let Some(parent) = &self.context.parent {
                            parent.report_child_failure(reason);
                        }
                    }
                }
            }

            self.scheduled.store(false, Ordering::Release);

            if !self.try_claim() {
                break;
            }

            let next = self.receiver.lock().unwrap().try_recv();
            match next {
                Ok(msg) => {
                    let mut actor = self.actor.lock().unwrap();
                    actor.receive(msg, &self.context);
                    continue;
                }
                Err(_) => {
                    self.scheduled.store(false, Ordering::Release);
                    break;
                }
            }
        }
    }
}

pub struct ActorRef<M: Send> {
    pub(crate) sender: Sender<M>,
    pub(crate) cell: Arc<dyn ScheduledWorker>,
    pub(crate) executor: Arc<Executor>,
    pub(crate) bus: Arc<DomainBus>,
    pub(crate) actor_id: ActorId,
}

impl<M: Send + 'static> ActorRef<M> {
    pub fn id(&self) -> ActorId {
        self.actor_id
    }

    pub fn tell(&self, message: M) {
        if self.sender.send(message).is_err() {
            if TypeId::of::<M>() != TypeId::of::<DeadLetter>() {
                self.bus.publish(DeadLetter {
                    message_type: std::any::type_name::<M>(),
                    actor_id: self.id(),
                });
            }
            return;
        }

        if self.cell.try_claim() {
            let cell = Arc::clone(&self.cell);
            self.executor.submit(move || cell.process_batch());
        }
    }

    pub fn erased(self) -> AnyActorRef {
        AnyActorRef { fail_hook: None }
    }
}

impl<M: Send> Clone for ActorRef<M> {
    fn clone(&self) -> Self {
        ActorRef {
            sender: self.sender.clone(),
            cell: Arc::clone(&self.cell),
            executor: Arc::clone(&self.executor),
            bus: Arc::clone(&self.bus),
            actor_id: self.actor_id,
        }
    }
}

#[derive(Clone)]
pub struct AnyActorRef {
    fail_hook: Option<Arc<dyn Fn(String) + Send + Sync>>,
}

impl AnyActorRef {
    pub fn report_child_failure(&self, reason: String) {
        if let Some(hook) = &self.fail_hook {
            hook(reason);
        }
    }
}
