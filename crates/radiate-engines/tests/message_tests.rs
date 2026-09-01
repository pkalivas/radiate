#[cfg(test)]
mod actor_tests {
    use radiate_core::Executor;
    use radiate_engines::message::{
        Actor, ActorContext, Addr, EventStream, Message, MessageHandler,
    };
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
        fn handle(&mut self, msg: &Add, _ctx: &ActorContext<Self>) {
            self.total += msg.0;
        }
    }

    impl MessageHandler<GetTotal> for Counter {
        fn handle(&mut self, _: &GetTotal, _ctx: &ActorContext<Self>) -> usize {
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
    impl Actor for SelfCounter {
        fn name(&self) -> &str {
            "SelfCounter"
        }
    }

    struct Tick;
    impl Message for Tick {
        type Response = ();
    }

    impl MessageHandler<Tick> for SelfCounter {
        fn handle(&mut self, _: &Tick, ctx: &ActorContext<Self>) {
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
    impl Actor for Emitter {
        fn name(&self) -> &str {
            "Emitter"
        }
    }
    impl MessageHandler<Ping> for Emitter {
        fn handle(&mut self, msg: &Ping, ctx: &ActorContext<Self>) {
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
    impl Actor for Listener {
        fn name(&self) -> &str {
            "Listener"
        }
    }
    impl MessageHandler<Ping> for Listener {
        fn handle(&mut self, msg: &Ping, _ctx: &ActorContext<Self>) {
            self.received.lock().unwrap().push(msg.clone());
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

    impl Message for Pong {
        type Response = ();
    }

    struct MultiListener {
        pings: Arc<AtomicUsize>,
        pongs: Arc<AtomicUsize>,
    }
    impl Actor for MultiListener {}
    impl MessageHandler<Ping> for MultiListener {
        fn handle(&mut self, _: &Ping, _ctx: &ActorContext<Self>) {
            self.pings.fetch_add(1, Ordering::SeqCst);
        }
    }
    impl MessageHandler<Pong> for MultiListener {
        fn handle(&mut self, _: &Pong, _ctx: &ActorContext<Self>) {
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
        fn handle(&mut self, _: &Boom, _ctx: &ActorContext<Self>) {
            panic!("simulated failure");
        }
    }
    impl MessageHandler<GetTotal> for Flaky {
        fn handle(&mut self, _: &GetTotal, _ctx: &ActorContext<Self>) -> usize {
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

#[cfg(test)]
mod event_stream_tests {
    use radiate_core::Executor;
    use radiate_engines::message::EventStream;
    use radiate_engines::message::Message;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

    #[derive(Debug, Clone, PartialEq)]
    struct Ping(u32);

    impl Message for Ping {
        type Response = ();
    }

    fn throughput(count: usize, elapsed: Duration) -> f64 {
        count as f64 / elapsed.as_secs_f64()
    }

    #[test]
    #[ignore = "throughput smoke test: cargo test --release -- --ignored --nocapture"]
    fn throughput_publish_single_subscriber_serial() {
        const N: usize = 500_000;
        let bus = EventStream::new(Arc::new(Executor::Serial));
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);

        bus.subscribe::<Ping>(move |_: &Ping| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        let start = Instant::now();
        for i in 0..N {
            bus.publish(Ping(i as u32));
        }
        let elapsed = start.elapsed();

        assert_eq!(count.load(Ordering::SeqCst), N);
        println!(
            "[publish/1 sub]     {N} events in {elapsed:?} = {:.0} events/sec",
            throughput(N, elapsed)
        );
    }

    #[test]
    #[ignore = "throughput smoke test: cargo test --release -- --ignored --nocapture"]
    fn throughput_publish_fanout_serial() {
        const N: usize = 100_000;
        const SUBSCRIBERS: usize = 8;

        let bus = EventStream::new(Arc::new(Executor::Serial));
        let counts: Vec<_> = (0..SUBSCRIBERS)
            .map(|_| Arc::new(AtomicUsize::new(0)))
            .collect();

        for count in &counts {
            let count = Arc::clone(count);
            bus.subscribe::<Ping>(move |_: &Ping| {
                count.fetch_add(1, Ordering::SeqCst);
            });
        }

        let start = Instant::now();
        for i in 0..N {
            bus.publish(Ping(i as u32));
        }
        let elapsed = start.elapsed();

        for count in &counts {
            assert_eq!(count.load(Ordering::SeqCst), N);
        }

        let deliveries = N * SUBSCRIBERS;
        println!(
            "[publish/fanout x{SUBSCRIBERS}] {deliveries} deliveries in {elapsed:?} = {:.0} deliveries/sec",
            throughput(deliveries, elapsed)
        );
    }

    #[test]
    #[ignore = "throughput smoke test: cargo test --release -- --ignored --nocapture"]
    fn throughput_publish_worker_pool() {
        const N: usize = 200_000;
        let bus = EventStream::new(Arc::new(Executor::FixedSizedWorkerPool(4)));
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);

        bus.subscribe::<Ping>(move |_: &Ping| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        let start = Instant::now();
        for i in 0..N {
            bus.publish(Ping(i as u32));
        }

        // publish() enqueues into the subscriber's own mailbox and
        // returns immediately on a pooled executor — poll for drain
        // completion instead of assuming publish() blocked.
        let deadline = Instant::now() + Duration::from_secs(30);
        while count.load(Ordering::SeqCst) < N {
            assert!(
                Instant::now() < deadline,
                "events not fully delivered in time"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        let elapsed = start.elapsed();

        println!(
            "[publish/pool]      {N} events in {elapsed:?} = {:.0} events/sec",
            throughput(N, elapsed)
        );
    }
}
