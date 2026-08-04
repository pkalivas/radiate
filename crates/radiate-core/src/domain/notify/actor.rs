use super::handler::EventHandler;
use super::message::Message;
use crate::{
    Envelope, Executor,
    notify::message::{ActorPanicked, AnyEnvelope, EventContext},
};
use radiate_utils::sentry_id;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::{
    any::{Any, TypeId},
    sync::mpsc::Receiver,
};
use std::{collections::HashMap, fmt};
use std::{collections::VecDeque, sync::mpsc::Sender};
use std::{
    panic::{self, AssertUnwindSafe},
    sync::RwLock,
};

sentry_id!(ActorId);

#[derive(Clone)]
pub struct ActorContext {
    executor: Arc<Executor>,
    parent: Option<AnyActorRef>,
    bus: Arc<DomainBus>,
}

pub trait Actor: Send {
    type Message: Send;
    fn receive(&mut self, message: Self::Message, ctx: &ActorContext);
    fn on_child_failure(&mut self, _reason: String);
}

trait ScheduledWorker: Send + Sync {
    fn try_claim(&self) -> bool;
    fn process_batch(self: Arc<Self>);
}

struct ActorCell<A: Actor> {
    actor: Arc<Mutex<A>>,
    receiver: Arc<Mutex<Receiver<A::Message>>>,
    scheduled: AtomicBool,
    context: ActorContext,
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
    sender: Sender<M>,
    cell: Arc<dyn ScheduledWorker>,
    executor: Arc<Executor>,
}

impl<M: Send> ActorRef<M> {
    pub fn tell(&self, message: M) {
        if self.sender.send(message).is_err() {
            return;
        }

        if self.cell.try_claim() {
            let cell = Arc::clone(&self.cell);
            self.executor.submit(move || cell.process_batch());
        }
    }

    pub fn erased(self) -> AnyActorRef {
        AnyActorRef {
            cell: Arc::clone(&self.cell),
            fail_hook: None,
        }
    }
}

impl<M: Send> Clone for ActorRef<M> {
    fn clone(&self) -> Self {
        ActorRef {
            sender: self.sender.clone(),
            cell: Arc::clone(&self.cell),
            executor: Arc::clone(&self.executor),
        }
    }
}

#[derive(Clone)]
pub struct AnyActorRef {
    cell: Arc<dyn ScheduledWorker>,
    fail_hook: Option<Arc<dyn Fn(String) + Send + Sync>>,
}

impl AnyActorRef {
    pub fn report_child_failure(&self, reason: String) {
        if let Some(hook) = &self.fail_hook {
            hook(reason);
        }
    }
}

pub struct ActorSystem {
    context: ActorContext,
}

impl ActorSystem {
    pub fn new(executor: Arc<Executor>) -> Self {
        ActorSystem {
            context: ActorContext {
                executor,
                parent: None,
                bus: Arc::new(DomainBus {
                    subscribers: RwLock::new(HashMap::new()),
                }),
            },
        }
    }

    pub fn has_subscribers<M: Message>(&self) -> bool {
        self.context.bus.has_subscribers::<M>()
    }

    pub fn publish<M: Send + 'static>(&self, message: M) {
        self.context.bus.publish(message);
    }

    pub fn spawn<A: Actor + 'static>(&self, actor: A) -> ActorRef<A::Message> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let cell = Arc::new(ActorCell {
            actor: Arc::new(Mutex::new(actor)),
            receiver: Arc::new(Mutex::new(receiver)),
            scheduled: AtomicBool::new(false),
            context: self.context.clone(),
        });

        ActorRef {
            sender,
            cell,
            executor: Arc::clone(&self.context.executor),
        }
    }

    pub fn subscribe<M: Send + Sync + 'static>(
        &self,
        handler: impl Fn(&M, &ActorContext) + Send + Sync + 'static,
    ) {
        let actor = FnActor {
            handler: Box::new(move |message: Envelope<M>, ctx: &ActorContext| {
                handler(&message, ctx);
            }),
        };

        let sub_ref = self.spawn(actor);
        self.context.bus.subscribe::<M>(sub_ref);
    }
}

pub struct FnActor<M: Send> {
    handler: Box<dyn Fn(M, &ActorContext) + Send + Sync>,
}

impl<M: Send> Actor for FnActor<M> {
    type Message = M;

    fn receive(&mut self, message: Self::Message, ctx: &ActorContext) {
        (self.handler)(message, ctx);
    }

    fn on_child_failure(&mut self, _reason: String) {}
}

#[derive(Default)]
pub struct DomainBus {
    subscribers: RwLock<HashMap<TypeId, Box<dyn AnySubscriber>>>,
}

impl DomainBus {
    pub fn new() -> Self {
        DomainBus {
            subscribers: RwLock::new(HashMap::new()),
        }
    }

    pub fn subscribe<M: Message>(&self, actor_ref: ActorRef<Envelope<M>>) {
        let mut registry = self.subscribers.write().unwrap();
        let group = registry
            .entry(TypeId::of::<M>())
            .or_insert_with(|| {
                Box::new(SubscriberGroup::<M> {
                    handles: Vec::new(),
                }) as Box<dyn AnySubscriber>
            })
            .as_any_mut()
            .downcast_mut::<SubscriberGroup<M>>()
            .expect("TypeId key always matches SubscriberGroup<M> by construction");
        group.handles.push(actor_ref);
    }

    pub fn publish<M: Send + 'static>(&self, message: M) {
        let registry = self.subscribers.read().unwrap();
        if let Some(group) = registry.get(&TypeId::of::<M>()) {
            group.dispatch(&Envelope::new(message));
        }
    }

    pub fn has_subscribers<M: Message>(&self) -> bool {
        self.subscribers
            .read()
            .unwrap()
            .contains_key(&TypeId::of::<M>())
    }
}

pub trait AnySubscriber: Send + Sync {
    fn type_name(&self) -> &'static str;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn dispatch(&self, envelope: &dyn Any);
}

struct SubscriberGroup<M: Send + Sync> {
    handles: Vec<ActorRef<Envelope<M>>>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use std::{
        sync::{Condvar, Mutex},
        time::Instant,
    };

    fn wait_for(signal: &Arc<(Mutex<usize>, Condvar)>, target: usize) {
        let (lock, cv) = &**signal;
        let mut count = lock.lock().unwrap();
        while *count < target {
            let (guard, timeout) = cv.wait_timeout(count, Duration::from_secs(2)).unwrap();
            count = guard;
            if timeout.timed_out() && *count < target {
                panic!("timed out waiting for {target} signals, saw {}", *count);
            }
        }
    }

    // ---------------------------------------------------------------
    // Bare Actor / ActorRef / ActorCell behavior
    // ---------------------------------------------------------------

    struct Counter {
        seen: Arc<Mutex<Vec<i32>>>,
        signal: Arc<(Mutex<usize>, Condvar)>,
    }

    impl Actor for Counter {
        type Message = i32;

        fn receive(&mut self, message: i32, _ctx: &ActorContext) {
            self.seen.lock().unwrap().push(message);
            let (count, cv) = &*self.signal;
            *count.lock().unwrap() += 1;
            cv.notify_all();
        }

        fn on_child_failure(&mut self, _reason: String) {}
    }

    #[test]
    fn tell_delivers_a_single_message() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        let actor_ref = system.spawn(Counter {
            seen: Arc::clone(&seen),
            signal: Arc::clone(&signal),
        });

        actor_ref.tell(7);
        wait_for(&signal, 1);

        assert_eq!(*seen.lock().unwrap(), vec![7]);
    }

    #[test]
    fn messages_are_delivered_in_fifo_order() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        let actor_ref = system.spawn(Counter {
            seen: Arc::clone(&seen),
            signal: Arc::clone(&signal),
        });

        const N: usize = 200;
        for i in 0..N as i32 {
            actor_ref.tell(i);
        }
        wait_for(&signal, N);

        let expected: Vec<i32> = (0..N as i32).collect();
        assert_eq!(*seen.lock().unwrap(), expected);
    }

    #[test]
    fn cloned_refs_all_feed_the_same_mailbox() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        let actor_ref = system.spawn(Counter {
            seen: Arc::clone(&seen),
            signal: Arc::clone(&signal),
        });

        let a = actor_ref.clone();
        let b = actor_ref.clone();
        let ha = std::thread::spawn(move || a.tell(1));
        let hb = std::thread::spawn(move || b.tell(2));
        ha.join().unwrap();
        hb.join().unwrap();

        wait_for(&signal, 2);

        let mut got = seen.lock().unwrap().clone();
        got.sort();
        assert_eq!(got, vec![1, 2]);
    }

    // ---------------------------------------------------------------
    // Panic isolation: a panicking message doesn't poison the actor
    // or stop subsequent messages from being processed.
    // ---------------------------------------------------------------

    struct Flaky {
        seen: Arc<Mutex<Vec<i32>>>,
        signal: Arc<(Mutex<usize>, Condvar)>,
    }

    impl Actor for Flaky {
        type Message = i32;

        fn receive(&mut self, message: i32, _ctx: &ActorContext) {
            if message == 2 {
                panic!("boom");
            }
            self.seen.lock().unwrap().push(message);
            let (count, cv) = &*self.signal;
            *count.lock().unwrap() += 1;
            cv.notify_all();
        }

        fn on_child_failure(&mut self, _reason: String) {}
    }

    #[test]
    fn actor_survives_a_panicking_message_and_keeps_processing() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        let actor_ref = system.spawn(Flaky {
            seen: Arc::clone(&seen),
            signal: Arc::clone(&signal),
        });

        actor_ref.tell(1);
        actor_ref.tell(2); // panics, should not take the actor down
        actor_ref.tell(3);

        wait_for(&signal, 2); // only 1 and 3 signal

        assert_eq!(*seen.lock().unwrap(), vec![1, 3]);
    }

    // ---------------------------------------------------------------
    // DomainBus: subscribe/publish fan-out
    // ---------------------------------------------------------------

    #[derive(Clone, Debug, PartialEq)]
    struct Counted(i32);

    #[derive(Clone, Debug, PartialEq)]
    struct Warning(&'static str);

    #[test]
    fn subscribe_and_publish_delivers_message() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        let seen2 = Arc::clone(&seen);
        let signal2 = Arc::clone(&signal);
        system.subscribe::<Counted>(move |msg, _ctx| {
            seen2.lock().unwrap().push(msg.0);
            let (count, cv) = &*signal2;
            *count.lock().unwrap() += 1;
            cv.notify_all();
        });

        system.publish(Counted(42));
        wait_for(&signal, 1);

        assert_eq!(*seen.lock().unwrap(), vec![42]);
    }

    #[test]
    fn unrelated_message_types_do_not_cross_wires() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        let seen2 = Arc::clone(&seen);
        let signal2 = Arc::clone(&signal);
        system.subscribe::<Counted>(move |msg, _ctx| {
            seen2.lock().unwrap().push(msg.0);
            let (count, cv) = &*signal2;
            *count.lock().unwrap() += 1;
            cv.notify_all();
        });

        // Nobody subscribed to Warning — should be a silent no-op.
        system.publish(Warning("disk almost full"));
        system.publish(Counted(1));
        wait_for(&signal, 1);

        assert_eq!(*seen.lock().unwrap(), vec![1]);
    }

    #[test]
    fn multiple_subscribers_of_same_type_all_receive() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let seen_a = Arc::new(Mutex::new(Vec::new()));
        let seen_b = Arc::new(Mutex::new(Vec::new()));
        let signal_a = Arc::new((Mutex::new(0), Condvar::new()));
        let signal_b = Arc::new((Mutex::new(0), Condvar::new()));

        let sa = Arc::clone(&seen_a);
        let siga = Arc::clone(&signal_a);
        system.subscribe::<Counted>(move |msg, _ctx| {
            sa.lock().unwrap().push(msg.0);
            let (count, cv) = &*siga;
            *count.lock().unwrap() += 1;
            cv.notify_all();
        });

        let sb = Arc::clone(&seen_b);
        let sigb = Arc::clone(&signal_b);
        system.subscribe::<Counted>(move |msg, _ctx| {
            sb.lock().unwrap().push(msg.0);
            let (count, cv) = &*sigb;
            *count.lock().unwrap() += 1;
            cv.notify_all();
        });

        system.publish(Counted(9));
        wait_for(&signal_a, 1);
        wait_for(&signal_b, 1);

        assert_eq!(*seen_a.lock().unwrap(), vec![9]);
        assert_eq!(*seen_b.lock().unwrap(), vec![9]);
    }

    #[test]
    fn publish_with_no_subscribers_does_not_panic() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        system.publish(Counted(1)); // should just be dropped
    }

    #[test]
    fn has_subscribers_reflects_registration_state() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        assert!(!system.has_subscribers::<Counted>());

        system.subscribe::<Counted>(|_msg, _ctx| {});

        assert!(system.has_subscribers::<Counted>());
        assert!(!system.has_subscribers::<Warning>());
    }

    // ---------------------------------------------------------------
    // AnyActorRef: erasure shouldn't break supervision no-ops
    // ---------------------------------------------------------------

    #[test]
    fn erased_ref_with_no_fail_hook_is_a_safe_no_op() {
        let system = ActorSystem::new(Arc::new(Executor::default()));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new((Mutex::new(0), Condvar::new()));

        let actor_ref = system.spawn(Counter {
            seen: Arc::clone(&seen),
            signal: Arc::clone(&signal),
        });

        let any_ref = actor_ref.erased();
        any_ref.report_child_failure("shouldn't panic".to_string());
    }

    fn wait_until<F: Fn() -> bool>(timeout: Duration, cond: F) -> bool {
        let start = Instant::now();
        while !cond() {
            if start.elapsed() > timeout {
                return false;
            }
            std::thread::yield_now();
        }
        true
    }

    #[test]
    fn fan_out_to_many_subscribers_throughput() {
        const N: u64 = 20_000;
        const SUBSCRIBERS: usize = 50;

        let system = ActorSystem::new(Arc::new(Executor::FixedSizedWorkerPool(4)));
        let total_received = Arc::new(AtomicU64::new(0));

        for _ in 0..SUBSCRIBERS {
            let total = Arc::clone(&total_received);
            system.subscribe::<Counted>(move |_msg: &Counted, _ctx: &ActorContext| {
                total.fetch_add(1, Ordering::Relaxed);
            });
        }

        let target = N * SUBSCRIBERS as u64;

        let start = Instant::now();
        for i in 0..N {
            system.publish(Counted(i as i32));
        }
        let ok = wait_until(Duration::from_secs(20), || {
            total_received.load(Ordering::Relaxed) == target
        });
        let elapsed = start.elapsed();

        assert!(
            ok,
            "timed out: expected {target} total deliveries, saw {}",
            total_received.load(Ordering::Relaxed)
        );

        let throughput = target as f64 / elapsed.as_secs_f64();
        println!(
            "[fan-out]    {N} messages x {SUBSCRIBERS} subscribers = {target} deliveries in {elapsed:?} ({throughput:.0} deliveries/sec)"
        );
    }
}

// type SubscriberRegistry = HashMap<TypeId, Vec<Box<dyn EventHandler

/// A single subscriber's mailbox. `tell` enqueues (message, context) pairs
/// and, if nobody is currently draining this actor, schedules a drain on the
/// executor. `scheduled` guarantees at most one in-flight drain per actor,
/// which is what gives every actor FIFO delivery and non-concurrent handling
/// regardless of how many worker threads the executor itself has.
///
/// The context is captured per-message at `tell` time (same as the
/// executor), not looked up fresh at drain time — `ActorSystem::set_sync`
/// only ever happens once, early, before real traffic starts, so this is
/// just the simplest thing that's still correct.
pub(super) struct Actor2<M: Message> {
    id: ActorId,
    handler: Mutex<Box<dyn EventHandler<M>>>,
    mailbox: Mutex<VecDeque<(Envelope<M>, EventContext)>>,
    scheduled: AtomicBool,
    num_processed: AtomicU64,
}

impl<M: Message> Actor2<M> {
    pub(super) fn new(handler: Box<dyn EventHandler<M>>) -> Arc<Self> {
        Arc::new(Actor2 {
            id: ActorId::new(),
            handler: Mutex::new(handler),
            mailbox: Mutex::new(VecDeque::new()),
            scheduled: AtomicBool::new(false),
            num_processed: AtomicU64::new(0),
        })
    }

    pub(super) fn id(&self) -> ActorId {
        self.id
    }

    pub(super) fn mailbox_len(&self) -> usize {
        self.mailbox.lock().unwrap().len()
    }

    pub(super) fn num_processed(&self) -> u64 {
        self.num_processed.load(Ordering::Acquire)
    }

    #[inline]
    pub(super) fn tell(
        self: &Arc<Self>,
        message: Envelope<M>,
        ctx: EventContext,
        executor: &Executor,
    ) {
        self.mailbox.lock().unwrap().push_back((message, ctx));

        if self
            .scheduled
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            let this = Arc::clone(self);
            executor.submit(move || this.drain());
        }
    }

    #[inline]
    fn drain(self: Arc<Self>) {
        loop {
            let batch = std::mem::take(&mut *self.mailbox.lock().unwrap());

            if batch.is_empty() {
                self.scheduled.store(false, Ordering::Release);

                let more_arrived = !self.mailbox.lock().unwrap().is_empty();
                if !more_arrived
                    || self
                        .scheduled
                        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                        .is_err()
                {
                    return;
                }
                continue;
            }

            let mut handler = self.handler.lock().unwrap();
            for (message, ctx) in batch {
                // `handler` is held by this frame, not by the closure below,
                // so if the closure panics and `catch_unwind` stops the
                // unwind right here, the guard is never dropped mid-unwind —
                // `self.handler`'s `Mutex` is never poisoned, and this actor
                // keeps handling the rest of the batch and every message
                // after it.
                let outcome =
                    panic::catch_unwind(AssertUnwindSafe(|| handler.handle(&*message, &ctx)));
                self.num_processed.fetch_add(1, Ordering::AcqRel);

                if let Err(payload) = outcome
                    && TypeId::of::<M>() != TypeId::of::<ActorPanicked>()
                {
                    ctx.send(ActorPanicked {
                        message_type: std::any::type_name::<M>(),
                        actor_id: self.id,
                        panic_message: panic_payload_to_string(payload),
                    });
                }
            }
        }
    }
}

fn panic_payload_to_string(payload: Box<dyn Any + Send>) -> String {
    if let Some(msg) = payload.downcast_ref::<&str>() {
        msg.to_string()
    } else if let Some(msg) = payload.downcast_ref::<String>() {
        msg.clone()
    } else {
        "actor handler panicked with a non-string payload".to_string()
    }
}

impl<M: Message> fmt::Debug for Actor2<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Actor")
            .field("id", &self.id)
            .field("message_type", &std::any::type_name::<M>())
            .field("scheduled", &self.scheduled.load(Ordering::Acquire))
            .field("mailbox_size", &self.mailbox.lock().unwrap().len())
            .field("num_processed", &self.num_processed.load(Ordering::Acquire))
            .finish()
    }
}

// use super::handler::EventHandler;
// use super::message::Message;
// use crate::{
//     Envelope, Executor,
//     notify::message::{ActorPanicked, EventContext},
// };
// use radiate_utils::sentry_id;
// use std::any::{Any, TypeId};
// use std::collections::VecDeque;
// use std::fmt;
// use std::panic::{self, AssertUnwindSafe};
// use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
// use std::sync::{Arc, Mutex};

// sentry_id!(ActorId);

// /// A single subscriber's mailbox. `tell` enqueues (message, context) pairs
// /// and, if nobody is currently draining this actor, schedules a drain on the
// /// executor. `scheduled` guarantees at most one in-flight drain per actor,
// /// which is what gives every actor FIFO delivery and non-concurrent handling
// /// regardless of how many worker threads the executor itself has.
// ///
// /// The context is captured per-message at `tell` time (same as the
// /// executor), not looked up fresh at drain time — `ActorSystem::set_sync`
// /// only ever happens once, early, before real traffic starts, so this is
// /// just the simplest thing that's still correct.
// pub(super) struct Actor<M: Message> {
//     id: ActorId,
//     handler: Mutex<Box<dyn EventHandler<M>>>,
//     mailbox: Mutex<VecDeque<(Envelope<M>, EventContext)>>,
//     scheduled: AtomicBool,
//     num_processed: AtomicU64,
// }

// impl<M: Message> Actor<M> {
//     pub(super) fn new(handler: Box<dyn EventHandler<M>>) -> Arc<Self> {
//         Arc::new(Actor {
//             id: ActorId::new(),
//             handler: Mutex::new(handler),
//             mailbox: Mutex::new(VecDeque::new()),
//             scheduled: AtomicBool::new(false),
//             num_processed: AtomicU64::new(0),
//         })
//     }

//     pub(super) fn id(&self) -> ActorId {
//         self.id
//     }

//     pub(super) fn mailbox_len(&self) -> usize {
//         self.mailbox.lock().unwrap().len()
//     }

//     pub(super) fn num_processed(&self) -> u64 {
//         self.num_processed.load(Ordering::Acquire)
//     }

//     #[inline]
//     pub(super) fn tell(
//         self: &Arc<Self>,
//         message: Envelope<M>,
//         ctx: EventContext,
//         executor: &Executor,
//     ) {
//         self.mailbox.lock().unwrap().push_back((message, ctx));

//         if self
//             .scheduled
//             .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
//             .is_ok()
//         {
//             let this = Arc::clone(self);
//             executor.submit(move || this.drain());
//         }
//     }

//     #[inline]
//     fn drain(self: Arc<Self>) {
//         loop {
//             let batch = std::mem::take(&mut *self.mailbox.lock().unwrap());

//             if batch.is_empty() {
//                 self.scheduled.store(false, Ordering::Release);

//                 let more_arrived = !self.mailbox.lock().unwrap().is_empty();
//                 if !more_arrived
//                     || self
//                         .scheduled
//                         .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
//                         .is_err()
//                 {
//                     return;
//                 }
//                 continue;
//             }

//             let mut handler = self.handler.lock().unwrap();
//             for (message, ctx) in batch {
//                 // `handler` is held by this frame, not by the closure below,
//                 // so if the closure panics and `catch_unwind` stops the
//                 // unwind right here, the guard is never dropped mid-unwind —
//                 // `self.handler`'s `Mutex` is never poisoned, and this actor
//                 // keeps handling the rest of the batch and every message
//                 // after it.
//                 let outcome =
//                     panic::catch_unwind(AssertUnwindSafe(|| handler.handle(&*message, &ctx)));
//                 self.num_processed.fetch_add(1, Ordering::AcqRel);

//                 if let Err(payload) = outcome
//                     && TypeId::of::<M>() != TypeId::of::<ActorPanicked>()
//                 {
//                     ctx.send(ActorPanicked {
//                         message_type: std::any::type_name::<M>(),
//                         actor_id: self.id,
//                         panic_message: panic_payload_to_string(payload),
//                     });
//                 }
//             }
//         }
//     }
// }

// fn panic_payload_to_string(payload: Box<dyn Any + Send>) -> String {
//     if let Some(msg) = payload.downcast_ref::<&str>() {
//         msg.to_string()
//     } else if let Some(msg) = payload.downcast_ref::<String>() {
//         msg.clone()
//     } else {
//         "actor handler panicked with a non-string payload".to_string()
//     }
// }

// impl<M: Message> fmt::Debug for Actor<M> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.debug_struct("Actor")
//             .field("id", &self.id)
//             .field("message_type", &std::any::type_name::<M>())
//             .field("scheduled", &self.scheduled.load(Ordering::Acquire))
//             .field("mailbox_size", &self.mailbox.lock().unwrap().len())
//             .field("num_processed", &self.num_processed.load(Ordering::Acquire))
//             .finish()
//     }
// }
