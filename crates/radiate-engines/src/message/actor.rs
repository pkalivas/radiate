use crate::message::{Event, EventStream};
use crossbeam::channel::{self, Receiver, Sender};
use radiate_core::{Executor, RadiateError, error::RadiateResult};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

pub trait Message: Send + Sync + 'static {
    type Response: Send + 'static;
}

impl<T: Event> Message for Arc<T> {
    type Response = ();
}

pub trait MessageHandler<M: Message>: Actor {
    fn handle(&mut self, msg: M, ctx: &ActorContext<Self>) -> M::Response
    where
        Self: Sized;
}

pub trait Actor: Send + Sync + 'static {
    fn started(&mut self, _ctx: &ActorContext<Self>)
    where
        Self: Sized,
    {
    }

    fn stopped(&mut self)
    where
        Self: Sized,
    {
    }
}

#[allow(dead_code)]
pub struct ActorContext<A>(Addr<A>);

impl<A: Actor> ActorContext<A> {
    pub fn send<M>(&self, msg: M)
    where
        A: MessageHandler<M>,
        M: Message,
    {
        (self.0).send(msg);
    }

    pub fn ask<M>(&self, msg: M) -> RadiateResult<M::Response>
    where
        A: MessageHandler<M>,
        M: Message,
    {
        (self.0).ask(msg)
    }

    pub fn publish<E: Event>(&self, message: E) {
        (self.0).publish(message);
    }
}

struct Mailbox<T> {
    sender: Sender<T>,
    receiver: Receiver<T>,
    scheduled: AtomicBool,
}

impl<T> Mailbox<T> {
    fn new() -> Self {
        let (sender, receiver) = channel::unbounded();
        Mailbox {
            sender,
            receiver,
            scheduled: AtomicBool::new(false),
        }
    }

    fn try_claim(&self) -> bool {
        self.scheduled
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    #[inline]
    fn drain(&self, mut process: impl FnMut(T)) {
        loop {
            while let Ok(item) = self.receiver.try_recv() {
                process(item);
            }

            self.scheduled.store(false, Ordering::Release);

            if !self.try_claim() {
                break;
            }

            let Some(item) = self.receiver.try_recv().ok() else {
                self.scheduled.store(false, Ordering::Release);
                break;
            };

            process(item);
        }
    }
}

type ProcessFn<A> = Box<dyn FnOnce(&mut A, &ActorContext<A>) + Send>;

struct Envelope<A> {
    run: ProcessFn<A>,
}

pub struct ActorCell<A> {
    actor: Mutex<A>,
    mailbox: Mailbox<Envelope<A>>,
    alive: AtomicBool,
}

impl<A: Actor> ActorCell<A> {
    fn new(actor: A) -> Self {
        ActorCell {
            actor: Mutex::new(actor),
            mailbox: Mailbox::new(),
            alive: AtomicBool::new(true),
        }
    }

    fn enqueue(&self, envelope: Envelope<A>) -> bool {
        if !self.alive.load(Ordering::Acquire) {
            return false;
        }

        self.mailbox.sender.send(envelope).is_ok()
    }

    fn process_batch(self: Arc<Self>, addr: Addr<A>) {
        let mut actor = self.actor.lock().unwrap_or_else(|e| e.into_inner());
        let mut stopping = false;

        self.mailbox.drain(|envelope| {
            if stopping {
                return;
            }

            let ctx = ActorContext(addr.clone());
            let result = catch_unwind(AssertUnwindSafe(|| (envelope.run)(&mut actor, &ctx)));

            if result.is_err() {
                self.alive.store(false, Ordering::Release);
                stopping = true;
            }
        });
    }
}

pub struct Addr<A> {
    cell: Arc<ActorCell<A>>,
    executor: Arc<Executor>,
    bus: Option<EventStream>,
}

impl<A: Actor> Addr<A> {
    pub fn spawn(actor: A, executor: Arc<Executor>) -> Self {
        Self::spawn_with_bus(actor, executor, None)
    }

    pub fn spawn_with_bus(actor: A, executor: Arc<Executor>, bus: Option<EventStream>) -> Self {
        let cell = Arc::new(ActorCell::new(actor));
        let addr = Addr {
            cell,
            executor,
            bus,
        };

        {
            let mut guard = addr.cell.actor.lock().unwrap_or_else(|e| e.into_inner());
            let ctx = ActorContext(addr.clone());

            guard.started(&ctx);
        }

        addr
    }

    pub fn send<M>(&self, msg: M)
    where
        A: MessageHandler<M>,
        M: Message,
    {
        let queued = self.cell.enqueue(Envelope {
            run: Box::new(move |actor: &mut A, ctx: &ActorContext<A>| {
                actor.handle(msg, ctx);
            }),
        });

        if queued {
            self.dispatch();
        }
    }

    pub fn ask<M>(&self, msg: M) -> RadiateResult<M::Response>
    where
        A: MessageHandler<M>,
        M: Message,
    {
        let (tx, rx) = channel::bounded(1);

        let queued = self.cell.enqueue(Envelope {
            run: Box::new(move |actor: &mut A, ctx: &ActorContext<A>| {
                let response = actor.handle(msg, ctx);
                // Caller may have dropped `rx` already (unlikely, but not
                // our problem if so) — don't let that panic the actor.
                let _ = tx.send(response);
            }),
        });

        if !queued {
            // Actor has stopped, so the caller will never get a response.
            return Err(RadiateError::Event(format!(
                "actor stopped before responding"
            )));
        }

        self.dispatch();
        rx.recv()
            .map_err(|_| RadiateError::Event(format!("Failed to receive response from actor")))
    }

    pub fn publish<E: Event>(&self, message: E) {
        if let Some(bus) = &self.bus {
            bus.publish(message);
        }
    }

    pub fn subscribe<E: Event>(&self) -> Option<crate::message::Subscription>
    where
        A: MessageHandler<Arc<E>>,
    {
        self.bus
            .as_ref()
            .map(|bus| bus.subscribe_addr::<E, A>(self))
    }

    fn dispatch(&self) {
        if self.cell.mailbox.try_claim() {
            let cell = Arc::clone(&self.cell);
            let addr = self.clone();
            self.executor.submit(move || cell.process_batch(addr));
        }
    }
}

impl<A> Clone for Addr<A> {
    fn clone(&self) -> Self {
        Addr {
            cell: Arc::clone(&self.cell),
            executor: Arc::clone(&self.executor),
            bus: self.bus.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    fn serial() -> Arc<Executor> {
        Arc::new(Executor::Serial)
    }

    // --- basic send/ask ---

    struct Counter {
        total: usize,
    }
    impl Actor for Counter {}

    struct Add(usize);
    impl Message for Add {
        type Response = ();
    }

    struct GetTotal;
    impl Message for GetTotal {
        type Response = usize;
    }

    impl MessageHandler<Add> for Counter {
        fn handle(&mut self, msg: Add, _ctx: &ActorContext<Self>) {
            self.total += msg.0;
        }
    }

    impl MessageHandler<GetTotal> for Counter {
        fn handle(&mut self, _: GetTotal, _ctx: &ActorContext<Self>) -> usize {
            self.total
        }
    }

    #[test]
    fn send_updates_actor_state_in_order() {
        let addr = Addr::spawn(Counter { total: 0 }, serial());
        addr.send(Add(3));
        addr.send(Add(4));
        assert_eq!(addr.ask(GetTotal).unwrap(), 7);
    }

    #[test]
    fn cloned_addr_shares_the_same_actor() {
        let addr = Addr::spawn(Counter { total: 0 }, serial());
        let other = addr.clone();
        addr.send(Add(1));
        other.send(Add(2));
        assert_eq!(addr.ask(GetTotal).unwrap(), 3);
    }

    #[test]
    fn ask_blocks_until_response_on_worker_pool() {
        let addr = Addr::spawn(
            Counter { total: 0 },
            Arc::new(Executor::FixedSizedWorkerPool(2)),
        );
        addr.send(Add(2));
        addr.send(Add(3));
        assert_eq!(addr.ask(GetTotal).unwrap(), 5);
    }

    // --- started() hook ---

    struct StartFlag {
        started: Arc<AtomicBool>,
    }
    impl Actor for StartFlag {
        fn started(&mut self, _ctx: &ActorContext<Self>) {
            self.started.store(true, Ordering::SeqCst);
        }
    }

    #[test]
    fn started_hook_runs_synchronously_during_spawn() {
        let flag = Arc::new(AtomicBool::new(false));
        let _addr = Addr::spawn(
            StartFlag {
                started: Arc::clone(&flag),
            },
            serial(),
        );
        // No message has been sent — if this is true, `started` ran as
        // part of `spawn` itself, not via the mailbox.
        assert!(flag.load(Ordering::SeqCst));
    }

    // --- self-sends re-enter the same drain, not a new dispatch ---

    struct SelfCounter {
        remaining: u32,
        total: Arc<AtomicUsize>,
    }
    impl Actor for SelfCounter {}

    struct Tick;
    impl Message for Tick {
        type Response = ();
    }

    impl MessageHandler<Tick> for SelfCounter {
        fn handle(&mut self, _: Tick, ctx: &ActorContext<Self>) {
            self.total.fetch_add(1, Ordering::SeqCst);
            if self.remaining > 0 {
                self.remaining -= 1;
                ctx.send(Tick);
            }
        }
    }

    #[test]
    fn actor_can_send_a_message_to_itself() {
        let total = Arc::new(AtomicUsize::new(0));
        let addr = Addr::spawn(
            SelfCounter {
                remaining: 4,
                total: Arc::clone(&total),
            },
            serial(),
        );

        addr.send(Tick);

        // 1 initial + 4 self-sends, all drained within the same
        // process_batch call rather than requiring separate dispatches.
        assert_eq!(total.load(Ordering::SeqCst), 5);
    }

    // --- actor <-> bus: publish from inside a handler ---

    #[derive(Debug, Clone, PartialEq)]
    struct Ping(u32);
    impl Message for Ping {
        type Response = ();
    }

    struct Emitter;
    impl Actor for Emitter {}
    impl MessageHandler<Ping> for Emitter {
        fn handle(&mut self, msg: Ping, ctx: &ActorContext<Self>) {
            ctx.publish(Ping(msg.0 * 10));
        }
    }

    #[test]
    fn actor_can_publish_to_the_bus_from_a_handler() {
        let executor = serial();
        let bus = EventStream::new(Arc::clone(&executor));
        let addr = Addr::spawn_with_bus(Emitter, Arc::clone(&executor), Some(bus.clone()));

        let received = Arc::new(Mutex::new(Vec::new()));
        let received_clone = Arc::clone(&received);
        bus.subscribe::<Ping>(move |event: &Ping| {
            received_clone.lock().unwrap().push(event.clone());
        });

        addr.send(Ping(3));

        assert_eq!(*received.lock().unwrap(), vec![Ping(30)]);
    }

    #[test]
    fn publish_without_a_bus_is_a_silent_noop() {
        let addr = Addr::spawn(Emitter, serial());
        // Emitter's handler calls ctx.publish — must not panic even
        // though this Addr has no bus attached.
        addr.send(Ping(1));
    }

    // --- actor <-> bus: subscribing directly ---

    struct Listener {
        received: Arc<Mutex<Vec<Ping>>>,
    }
    impl Actor for Listener {}
    impl MessageHandler<Arc<Ping>> for Listener {
        fn handle(&mut self, msg: Arc<Ping>, _ctx: &ActorContext<Self>) {
            self.received.lock().unwrap().push((*msg).clone());
        }
    }

    #[test]
    fn actor_subscribes_directly_to_bus_events() {
        let executor = serial();
        let bus = EventStream::new(Arc::clone(&executor));
        let received = Arc::new(Mutex::new(Vec::new()));
        let listener = Addr::spawn_with_bus(
            Listener {
                received: Arc::clone(&received),
            },
            Arc::clone(&executor),
            Some(bus.clone()),
        );

        listener.subscribe::<Ping>();
        bus.publish(Ping(7));

        assert_eq!(*received.lock().unwrap(), vec![Ping(7)]);
    }

    #[test]
    fn subscribe_without_a_bus_returns_none() {
        let addr = Addr::spawn(
            Listener {
                received: Arc::new(Mutex::new(Vec::new())),
            },
            serial(),
        );
        assert!(addr.subscribe::<Ping>().is_none());
    }

    // --- one actor, multiple event types: the case that would've hit
    // E0119 under the old `impl<A, M> Trait for Addr<A>` design ---

    #[derive(Debug, Clone, PartialEq)]
    struct Pong(u32);

    struct MultiListener {
        pings: Arc<AtomicUsize>,
        pongs: Arc<AtomicUsize>,
    }
    impl Actor for MultiListener {}
    impl MessageHandler<Arc<Ping>> for MultiListener {
        fn handle(&mut self, _: Arc<Ping>, _ctx: &ActorContext<Self>) {
            self.pings.fetch_add(1, Ordering::SeqCst);
        }
    }
    impl MessageHandler<Arc<Pong>> for MultiListener {
        fn handle(&mut self, _: Arc<Pong>, _ctx: &ActorContext<Self>) {
            self.pongs.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn actor_subscribes_to_multiple_event_types() {
        let executor = serial();
        let bus = EventStream::new(Arc::clone(&executor));
        let pings = Arc::new(AtomicUsize::new(0));
        let pongs = Arc::new(AtomicUsize::new(0));
        let addr = Addr::spawn_with_bus(
            MultiListener {
                pings: Arc::clone(&pings),
                pongs: Arc::clone(&pongs),
            },
            Arc::clone(&executor),
            Some(bus.clone()),
        );

        addr.subscribe::<Ping>();
        addr.subscribe::<Pong>();

        bus.publish(Ping(1));
        bus.publish(Pong(2));
        bus.publish(Ping(3));

        assert_eq!(pings.load(Ordering::SeqCst), 2);
        assert_eq!(pongs.load(Ordering::SeqCst), 1);
    }

    struct Flaky {
        total: usize,
    }
    impl Actor for Flaky {}

    struct Boom;
    impl Message for Boom {
        type Response = ();
    }
    impl MessageHandler<Boom> for Flaky {
        fn handle(&mut self, _: Boom, _ctx: &ActorContext<Self>) {
            panic!("simulated failure");
        }
    }
    impl MessageHandler<GetTotal> for Flaky {
        fn handle(&mut self, _: GetTotal, _ctx: &ActorContext<Self>) -> usize {
            self.total
        }
    }

    #[test]
    fn ask_after_actor_stopped_returns_err_instead_of_hanging() {
        let addr = Addr::spawn(Flaky { total: 0 }, serial());

        addr.send(Boom); // default on_panic() is Stop — actor is now dead

        // Without the fix, this would block forever waiting on a reply
        // that will never come. With it, enqueue sees `alive == false`
        // and returns Disconnected immediately.
        assert!(addr.ask(GetTotal).is_err());
    }

    use std::time::{Duration, Instant};

    fn throughput(count: usize, elapsed: Duration) -> f64 {
        count as f64 / elapsed.as_secs_f64()
    }

    // --- single actor, fire-and-forget, no async dispatch in the loop ---

    #[test]
    #[ignore = "throughput smoke test: cargo test --release -- --ignored --nocapture"]
    fn throughput_send_serial() {
        const N: usize = 1_000_000;
        let addr = Addr::spawn(Counter { total: 0 }, serial());

        let start = Instant::now();
        for _ in 0..N {
            addr.send(Add(1));
        }
        let elapsed = start.elapsed();

        // Executor::Serial runs process_batch inline during dispatch, so by
        // the time the loop above returns, everything's already processed —
        // this ask is a correctness check, not a completion barrier.
        assert_eq!(addr.ask(GetTotal).unwrap(), N);

        println!(
            "[send/serial]   {N} msgs in {elapsed:?} = {:.0} msgs/sec",
            throughput(N, elapsed)
        );
    }

    // --- single actor, pooled executor: measures dispatch/scheduling
    // overhead, NOT parallelism — one actor's mailbox is still drained
    // by exactly one thread at a time regardless of pool size ---

    #[test]
    #[ignore = "throughput smoke test: cargo test --release -- --ignored --nocapture"]
    fn throughput_send_worker_pool() {
        const N: usize = 1_000_000;
        let executor = Arc::new(Executor::FixedSizedWorkerPool(4));
        let addr = Addr::spawn(Counter { total: 0 }, executor);

        let start = Instant::now();
        for _ in 0..N {
            addr.send(Add(1));
        }
        // The channel is FIFO and this is the only producer thread, so this
        // ask can only return after every prior Add has drained — it's the
        // completion barrier the serial test didn't need.
        let total = addr.ask(GetTotal).unwrap();
        let elapsed = start.elapsed();

        assert_eq!(total, N);
        println!(
            "[send/pool]     {N} msgs in {elapsed:?} = {:.0} msgs/sec",
            throughput(N, elapsed)
        );
    }

    // --- many actors, pooled executor, concurrent producers: the test
    // that actually exercises cross-actor parallelism ---

    #[test]
    #[ignore = "throughput smoke test: cargo test --release -- --ignored --nocapture"]
    fn throughput_many_actors_parallel() {
        const ACTORS: usize = 8;
        const PER_ACTOR: usize = 200_000;

        let executor = Arc::new(Executor::FixedSizedWorkerPool(8));
        let addrs: Vec<_> = (0..ACTORS)
            .map(|_| Addr::spawn(Counter { total: 0 }, Arc::clone(&executor)))
            .collect();

        let start = Instant::now();
        std::thread::scope(|scope| {
            for addr in &addrs {
                scope.spawn(|| {
                    for _ in 0..PER_ACTOR {
                        addr.send(Add(1));
                    }
                });
            }
        });

        for addr in &addrs {
            assert_eq!(addr.ask(GetTotal).unwrap(), PER_ACTOR);
        }
        let elapsed = start.elapsed();

        let total = ACTORS * PER_ACTOR;
        println!(
            "[send/parallel] {ACTORS} actors x {PER_ACTOR} = {total} msgs in {elapsed:?} = {:.0} msgs/sec",
            throughput(total, elapsed)
        );
    }

    // --- round-trip cost: bounded channel + block-on-recv per call ---

    #[test]
    #[ignore = "throughput smoke test: cargo test --release -- --ignored --nocapture"]
    fn throughput_ask_roundtrip_serial() {
        const N: usize = 200_000;
        let addr = Addr::spawn(Counter { total: 0 }, serial());

        let start = Instant::now();
        for _ in 0..N {
            addr.ask(GetTotal).unwrap();
        }
        let elapsed = start.elapsed();

        println!(
            "[ask/serial]    {N} round-trips in {elapsed:?} = {:.0} asks/sec",
            throughput(N, elapsed)
        );
    }
}
