#[cfg(test)]
mod event_stream_tests {
    use radiate_core::{Executor, error::RadiateResult};
    use radiate_engines::{
        Event,
        events::{EventContext, EventHandler, EventStream, Handler},
    };
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn serial() -> Arc<Executor> {
        Arc::new(Executor::Serial)
    }

    struct Add(usize);

    impl Event for Add {}

    struct Counter {
        total: Arc<AtomicUsize>,
    }

    impl Handler<Add> for Counter {
        fn handle(&mut self, event: &Add, _ctx: &EventContext<'_, Self>) {
            self.total.fetch_add(event.0, Ordering::SeqCst);
        }
    }

    // --- basic subscribe/publish ---

    #[test]
    fn subscribe_and_publish_round_trips_through_a_stateful_handler() {
        let stream = EventStream::new(serial());
        let total = Arc::new(AtomicUsize::new(0));

        stream.subscribe(Counter {
            total: Arc::clone(&total),
        });

        stream.publish(Add(3));
        stream.publish(Add(4));

        assert_eq!(total.load(Ordering::SeqCst), 7);
    }

    #[test]
    fn a_plain_closure_is_a_handler_via_the_blanket_impl() {
        let stream = EventStream::new(serial());
        let total = Arc::new(AtomicUsize::new(0));
        let total_clone = Arc::clone(&total);

        stream.subscribe(move |event: &Add| {
            total_clone.fetch_add(event.0, Ordering::SeqCst);
        });

        stream.publish(Add(5));
        stream.publish(Add(6));

        assert_eq!(total.load(Ordering::SeqCst), 11);
    }

    #[test]
    fn publish_fans_out_to_every_subscriber() {
        let stream = EventStream::new(serial());
        let a_total = Arc::new(AtomicUsize::new(0));
        let b_total = Arc::new(AtomicUsize::new(0));

        stream.subscribe({
            let total = Arc::clone(&a_total);
            move |event: &Add| {
                total.fetch_add(event.0, Ordering::SeqCst);
            }
        });
        stream.subscribe({
            let total = Arc::clone(&b_total);
            move |event: &Add| {
                total.fetch_add(event.0, Ordering::SeqCst);
            }
        });

        stream.publish(Add(3));
        stream.publish(Add(4));

        assert_eq!(a_total.load(Ordering::SeqCst), 7);
        assert_eq!(b_total.load(Ordering::SeqCst), 7);
    }

    #[test]
    fn publish_with_no_subscribers_is_a_noop() {
        let stream = EventStream::new(serial());
        stream.publish(Add(1)); // must not panic
    }

    // --- reacting by publishing back onto the stream ---

    #[derive(Clone, Debug)]
    struct Doubled(usize);

    impl Event for Doubled {}

    struct Doubler;

    impl Handler<Add> for Doubler {
        fn handle(&mut self, event: &Add, ctx: &EventContext<'_, Self>) {
            ctx.publish(Doubled(event.0 * 2));
        }
    }

    #[test]
    fn a_handler_can_react_by_publishing_a_new_event_type() {
        let stream = EventStream::new(serial());
        let doubled_total = Arc::new(AtomicUsize::new(0));

        stream.subscribe(Doubler);
        stream.subscribe({
            let total = Arc::clone(&doubled_total);
            move |event: &Doubled| {
                total.fetch_add(event.0, Ordering::SeqCst);
            }
        });

        stream.publish(Add(3));
        stream.publish(Add(4));

        assert_eq!(doubled_total.load(Ordering::SeqCst), 14);
    }

    // --- subscribing to more than one event type ---

    #[allow(dead_code)]
    struct Ping(u32);

    impl Event for Ping {}

    #[allow(dead_code)]
    struct Pong(u32);

    impl Event for Pong {}

    struct Both {
        pings: Arc<AtomicUsize>,
        pongs: Arc<AtomicUsize>,
    }

    impl EventHandler for Both {
        fn start(&mut self, ctx: &EventContext<'_, Self>) -> RadiateResult<()> {
            ctx.subscribe::<Ping>();
            ctx.subscribe::<Pong>();
            Ok(())
        }
    }

    impl Handler<Ping> for Both {
        fn handle(&mut self, _: &Ping, _ctx: &EventContext<'_, Self>) {
            self.pings.fetch_add(1, Ordering::SeqCst);
        }
    }

    impl Handler<Pong> for Both {
        fn handle(&mut self, _: &Pong, _ctx: &EventContext<'_, Self>) {
            self.pongs.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn event_handler_on_start_wires_up_more_than_one_event_type() {
        let stream = EventStream::new(serial());
        let pings = Arc::new(AtomicUsize::new(0));
        let pongs = Arc::new(AtomicUsize::new(0));

        stream
            .attach(Both {
                pings: Arc::clone(&pings),
                pongs: Arc::clone(&pongs),
            })
            .unwrap();

        stream.publish(Ping(1));
        stream.publish(Pong(2));
        stream.publish(Ping(3));

        assert_eq!(pings.load(Ordering::SeqCst), 2);
        assert_eq!(pongs.load(Ordering::SeqCst), 1);
    }

    // --- unsubscribe, scheduling ---

    #[test]
    fn unsubscribe_stops_further_delivery() {
        let stream = EventStream::new(serial());
        let total = Arc::new(AtomicUsize::new(0));

        let subscription = stream.subscribe({
            let total = Arc::clone(&total);
            move |_: &Ping| {
                total.fetch_add(1, Ordering::SeqCst);
            }
        });

        stream.publish(Ping(1));
        subscription.unsubscribe();
        stream.publish(Ping(1));

        assert_eq!(total.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn stream_unsubscribe_by_id_stops_further_delivery() {
        let stream = EventStream::new(serial());
        let total = Arc::new(AtomicUsize::new(0));

        let subscription = stream.subscribe({
            let total = Arc::clone(&total);
            move |_: &Ping| {
                total.fetch_add(1, Ordering::SeqCst);
            }
        });

        stream.publish(Ping(1));
        stream.unsubscribe(subscription.id());
        stream.publish(Ping(1));

        assert_eq!(total.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn lazy_publish_skips_construction_with_no_subscribers() {
        let stream = EventStream::new(serial());
        let built = Arc::new(AtomicUsize::new(0));
        let built_clone = Arc::clone(&built);

        stream
            .lazy_publish(move || {
                built_clone.fetch_add(1, Ordering::SeqCst);
                Ping(1)
            })
            .unwrap();

        assert_eq!(built.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn every_n_schedule_throttles_lazy_publish_delivery() {
        let stream = EventStream::new(serial());
        let total = Arc::new(AtomicUsize::new(0));

        stream
            .subscribe({
                let total = Arc::clone(&total);
                move |_: &Ping| {
                    total.fetch_add(1, Ordering::SeqCst);
                }
            })
            .schedule(3_usize)
            .unwrap();

        for i in 0..6 {
            stream.lazy_publish(move || Ping(i)).unwrap();
        }

        // Due on the 3rd and 6th publish only.
        assert_eq!(total.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn subscribers_on_the_same_event_keep_independent_schedules() {
        let stream = EventStream::new(serial());
        let every_two = stream.subscribe(|_: &Ping| {});
        let every_three = stream.subscribe(|_: &Ping| {});
        every_two.schedule(2_usize).unwrap();
        every_three.schedule(3_usize).unwrap();

        for i in 0..6 {
            stream.lazy_publish(move || Ping(i)).unwrap();
        }

        // An earlier subscriber being due must not stop later ones from ticking.
        let two = every_two.stats();
        let three = every_three.stats();
        assert_eq!((two.delivered, two.skipped), (3, 3));
        assert_eq!((three.delivered, three.skipped), (2, 4));
    }

    // --- subscription stats ---

    #[test]
    fn stats_count_delivered_and_handled_events() {
        let stream = EventStream::new(serial());
        let subscription = stream.subscribe(|_: &Ping| {}).named("pings");

        for i in 0..4 {
            stream.publish(Ping(i));
        }

        let stats = subscription.stats();
        assert_eq!(stats.delivered, 4);
        assert_eq!(stats.handled, 4);
        assert_eq!(stats.skipped, 0);
        assert_eq!(stats.in_flight(), 0);
    }

    #[test]
    fn stream_stats_report_live_subscriptions_by_name_and_event() {
        let stream = EventStream::new(serial());
        let pings = stream.subscribe(|_: &Ping| {}).named("pings");
        let adds = stream.subscribe(|_: &Add| {});

        stream.publish(Ping(1));
        stream.publish(Add(1));
        stream.publish(Add(2));

        let report = stream.stats();
        assert_eq!(report.subscribers.len(), 2);

        let ping_info = report
            .subscribers
            .iter()
            .find(|info| info.id == pings.id())
            .unwrap();
        assert_eq!(ping_info.name.as_ref().map(|n| n.as_str()), Some("pings"));
        assert!(ping_info.event.ends_with("Ping"));
        assert_eq!(ping_info.stats.delivered, 1);

        let add_info = report
            .subscribers
            .iter()
            .find(|info| info.id == adds.id())
            .unwrap();
        assert_eq!(add_info.name, None);
        assert_eq!(add_info.stats.delivered, 2);

        adds.unsubscribe();
        let report = stream.stats();
        assert_eq!(report.subscribers.len(), 1);
        assert_eq!(report.subscribers[0].id, pings.id());
    }

    // --- the explicit design choice: panics propagate, they aren't isolated ---

    #[test]
    fn a_panicking_handler_poisons_the_lock_instead_of_being_isolated() {
        struct Boom;

        impl Handler<Add> for Boom {
            fn handle(&mut self, _: &Add, _ctx: &EventContext<'_, Self>) {
                panic!("simulated failure");
            }
        }

        let stream = EventStream::new(serial());
        stream.subscribe(Boom);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            stream.publish(Add(1));
        }));
        assert!(result.is_err());

        // Every subsequent delivery to that same subscriber panics too — cascading rather
        // than being quietly isolated, per this design's stance that a panicking handler
        // should stop the run, not be walled off.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            stream.publish(Add(1));
        }));

        assert!(result.is_err());
    }

    // --- pooled executor: dispatch can run off the publisher's thread ---

    #[test]
    fn a_pooled_executor_can_run_the_handler_off_the_publisher_thread() {
        let stream = EventStream::new(Arc::new(Executor::FixedSizedWorkerPool(4)));
        let publisher = std::thread::current().id();
        let (tx, rx) = std::sync::mpsc::channel();

        stream.subscribe(move |_: &Ping| {
            tx.send(std::thread::current().id()).unwrap();
        });

        stream.publish(Ping(1));

        let handler_thread = rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
        assert_ne!(handler_thread, publisher);
    }

    // --- throughput smoke tests: run with
    //   cargo test -p radiate-engines --release -- --ignored --nocapture

    use std::time::{Duration, Instant};

    fn throughput(count: usize, elapsed: Duration) -> f64 {
        count as f64 / elapsed.as_secs_f64()
    }

    #[test]
    #[ignore = "throughput smoke test: cargo test --release -- --ignored --nocapture"]
    fn throughput_publish_single_subscriber() {
        const WARMUP: usize = 50_000;
        const N: usize = 500_000;

        let stream = EventStream::new(serial());
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);

        stream.subscribe(move |_: &Ping| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        for i in 0..WARMUP {
            stream.publish(Ping(i as u32));
        }

        let baseline = count.load(Ordering::SeqCst);

        let start = Instant::now();
        for i in 0..N {
            stream.publish(Ping(i as u32));
        }
        let elapsed = start.elapsed();

        assert_eq!(count.load(Ordering::SeqCst) - baseline, N);
        println!(
            "[stream/publish 1 sub]   {N} events in {elapsed:?} = {:.0} events/sec",
            throughput(N, elapsed)
        );
    }

    // [stream/publish fanout x8] 800000 deliveries in 112.598375ms = 7,104,898 deliveries/sec
    // [stream/publish fanout x8] 800000 deliveries in 113.196709ms = 7,067,343 deliveries/sec
    // [stream/publish fanout x8] 800000 deliveries in 112.595708ms = 7,105,067 deliveries/sec
    // [stream/publish fanout x8] 800000 deliveries in 354.411792ms = 2,257,261 deliveries/sec
    // [stream/publish fanout x8] 800000 deliveries in 351.081541ms = 2,278,673 deliveries/sec
    #[test]
    #[ignore = "throughput smoke test: cargo test --release -- --ignored --nocapture"]
    fn throughput_publish_fanout() {
        const WARMUP: usize = 20_000;
        const N: usize = 100_000;
        const SUBSCRIBERS: usize = 8;

        let stream = EventStream::new(serial());
        let counts: Vec<_> = (0..SUBSCRIBERS)
            .map(|_| Arc::new(AtomicUsize::new(0)))
            .collect();

        for count in &counts {
            let count = Arc::clone(count);
            stream.subscribe(move |_: &Ping| {
                count.fetch_add(1, Ordering::SeqCst);
            });
        }

        for i in 0..WARMUP {
            stream.publish(Ping(i as u32));
        }

        let baseline = counts[0].load(Ordering::SeqCst);

        let start = Instant::now();
        for i in 0..N {
            stream.publish(Ping(i as u32));
        }
        let elapsed = start.elapsed();

        for count in &counts {
            assert_eq!(count.load(Ordering::SeqCst) - baseline, N);
        }

        let deliveries = N * SUBSCRIBERS;
        println!(
            "[stream/publish fanout x{SUBSCRIBERS}] {deliveries} deliveries in {elapsed:?} = {:.0} deliveries/sec",
            throughput(deliveries, elapsed)
        );
    }

    // [stream/publish pool]    200000 events in 288.747291ms = 692,647 events/sec
    // [stream/publish pool]    200000 events in 281.040792ms = 711,640 events/sec
    // [stream/publish pool]    200000 events in 289.552166ms = 690,722 events/sec
    // [stream/publish pool]    200000 events in 254.11675ms = 787,040 events/sec
    // [stream/publish pool]    200000 events in 246.222ms = 812,275 events/sec
    #[test]
    #[ignore = "throughput smoke test: cargo test --release -- --ignored --nocapture"]
    fn throughput_publish_worker_pool() {
        const WARMUP: usize = 20_000;
        const N: usize = 200_000;
        let stream = EventStream::new(Arc::new(Executor::FixedSizedWorkerPool(4)));
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);

        stream.subscribe(move |_: &Ping| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        for i in 0..WARMUP {
            stream.publish(Ping(i as u32));
        }
        let warmup_deadline = Instant::now() + Duration::from_secs(30);
        while count.load(Ordering::SeqCst) < WARMUP {
            assert!(
                Instant::now() < warmup_deadline,
                "warmup did not drain in time"
            );
            std::thread::yield_now();
        }
        let baseline = count.load(Ordering::SeqCst);

        let start = Instant::now();
        for i in 0..N {
            stream.publish(Ping(i as u32));
        }

        let deadline = Instant::now() + Duration::from_secs(30);
        while count.load(Ordering::SeqCst) - baseline < N {
            assert!(
                Instant::now() < deadline,
                "events not fully delivered in time"
            );
            std::thread::yield_now();
        }
        let elapsed = start.elapsed();

        println!(
            "[stream/publish pool]    {N} events in {elapsed:?} = {:.0} events/sec",
            throughput(N, elapsed)
        );
    }
}

#[cfg(test)]
mod trace_tests {
    use radiate_core::Executor;
    use radiate_engines::events::{
        DeferredEvent, Event, EventContext, EventHandler, EventId, EventStream, Handler,
    };
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc::{self, RecvTimeoutError};
    use std::time::Duration;

    // Long enough that a passing run never hits it; only a hang or regression does.
    const GENEROUS: Duration = Duration::from_secs(5);
    // How long to watch for something that must NOT happen yet.
    const BRIEF: Duration = Duration::from_millis(50);

    struct Ping;
    struct Pong;

    impl Event for Ping {}
    impl Event for Pong {}

    /// Turns each deferred Ping into a Pong, published through the handler's context.
    struct Relay;

    impl EventHandler for Relay {
        fn start(
            &mut self,
            ctx: &EventContext<'_, Self>,
        ) -> radiate_core::error::RadiateResult<()> {
            ctx.subscribe::<DeferredEvent<Ping>>();
            Ok(())
        }
    }

    impl Handler<DeferredEvent<Ping>> for Relay {
        fn handle(&mut self, _: &DeferredEvent<Ping>, ctx: &EventContext<'_, Self>) {
            ctx.publish(Pong);
        }
    }

    /// A handler for `E` that signals when it starts, then blocks until released.
    fn gated_handler<E: Event>(
        stream: &EventStream,
    ) -> (mpsc::Receiver<()>, mpsc::Sender<()>, Arc<AtomicBool>) {
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel::<()>();
        let finished = Arc::new(AtomicBool::new(false));

        let done = Arc::clone(&finished);
        stream.subscribe(move |_: &E| {
            started_tx.send(()).unwrap();
            release_rx.recv().unwrap();
            done.store(true, Ordering::SeqCst);
        });

        (started_rx, release_tx, finished)
    }

    fn flush_on_thread(stream: &EventStream) -> mpsc::Receiver<()> {
        let (flushed_tx, flushed_rx) = mpsc::channel();
        let stream = stream.clone();
        std::thread::spawn(move || {
            stream.flush();
            flushed_tx.send(()).unwrap();
        });
        flushed_rx
    }

    #[test]
    fn flush_waits_for_events_published_by_deferred_handlers() {
        let stream = EventStream::from(Executor::FixedSizedWorkerPool(4));
        stream.attach(Relay).unwrap();
        let (started, release, finished) = gated_handler::<Pong>(&stream);

        stream.defer(Ping);
        let flushed = flush_on_thread(&stream);

        // The Pong handler (one hop from the deferred Ping) is running and blocked...
        started.recv_timeout(GENEROUS).unwrap();
        // ...so flush must still be waiting.
        assert_eq!(flushed.recv_timeout(BRIEF), Err(RecvTimeoutError::Timeout));

        release.send(()).unwrap();
        flushed.recv_timeout(GENEROUS).unwrap();
        assert!(finished.load(Ordering::SeqCst));
    }

    #[test]
    fn flush_does_not_wait_on_plain_publishes() {
        struct Block;
        impl Event for Block {}

        let stream = EventStream::from(Executor::FixedSizedWorkerPool(4));
        stream.attach(Relay).unwrap();
        let (started, release, finished) = gated_handler::<Block>(&stream);

        // A plain publish, outside any chain, blocks a worker...
        stream.publish(Block);
        started.recv_timeout(GENEROUS).unwrap();

        // ...but flushing an unrelated deferred event must not wait for it.
        stream.defer(Ping);
        let flushed = flush_on_thread(&stream);
        flushed.recv_timeout(GENEROUS).unwrap();
        assert!(!finished.load(Ordering::SeqCst));

        release.send(()).unwrap();
    }

    #[test]
    fn events_in_a_chain_share_the_root_id() {
        struct Recorder(mpsc::Sender<EventId>);

        impl EventHandler for Recorder {
            fn start(
                &mut self,
                ctx: &EventContext<'_, Self>,
            ) -> radiate_core::error::RadiateResult<()> {
                ctx.subscribe::<DeferredEvent<Ping>>();
                ctx.subscribe::<Pong>();
                Ok(())
            }
        }

        impl Handler<DeferredEvent<Ping>> for Recorder {
            fn handle(&mut self, _: &DeferredEvent<Ping>, ctx: &EventContext<'_, Self>) {
                self.0.send(ctx.root_id()).unwrap();
                ctx.publish(Pong);
            }
        }

        impl Handler<Pong> for Recorder {
            fn handle(&mut self, _: &Pong, ctx: &EventContext<'_, Self>) {
                self.0.send(ctx.root_id()).unwrap();
            }
        }

        let (tx, rx) = mpsc::channel();
        let stream = EventStream::from(Executor::FixedSizedWorkerPool(2));
        stream.attach(Recorder(tx)).unwrap();

        stream.defer(Ping);
        stream.flush();
        stream.publish(Pong);

        let ping = rx.recv_timeout(GENEROUS).unwrap();
        let chained_pong = rx.recv_timeout(GENEROUS).unwrap();
        let plain_pong = rx.recv_timeout(GENEROUS).unwrap();

        assert_eq!(
            ping, chained_pong,
            "a Pong published by the Ping handler joins its chain"
        );
        assert_ne!(
            chained_pong, plain_pong,
            "a plain publish starts a new chain"
        );
    }
}

#[cfg(test)]
mod mailbox_tests {
    use radiate_core::Executor;
    use radiate_core::error::RadiateResult;
    use radiate_engines::events::{
        DeferredEvent, Event, EventContext, EventHandler, EventStream, Handler,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex, mpsc};
    use std::time::Duration;

    // Long enough that a passing run never hits it; only a hang or regression does.
    const GENEROUS: Duration = Duration::from_secs(5);

    /// Run `f` on its own thread and fail (instead of hanging the suite) if it
    /// doesn't finish. These tests cover paths that used to deadlock.
    fn within<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || tx.send(f()).unwrap());
        rx.recv_timeout(GENEROUS)
            .expect("timed out: likely deadlocked")
    }

    struct Tick(u32);
    impl Event for Tick {}

    /// Counts down by publishing a smaller Tick to itself, recording when each
    /// handle call starts and ends.
    struct Countdown(Arc<Mutex<Vec<String>>>);

    impl EventHandler for Countdown {
        fn start(&mut self, ctx: &EventContext<'_, Self>) -> RadiateResult<()> {
            ctx.subscribe::<Tick>();
            Ok(())
        }
    }

    impl Handler<Tick> for Countdown {
        fn handle(&mut self, event: &Tick, ctx: &EventContext<'_, Self>) {
            self.0.lock().unwrap().push(format!("enter {}", event.0));
            if event.0 > 0 {
                ctx.publish(Tick(event.0 - 1));
            }
            self.0.lock().unwrap().push(format!("exit {}", event.0));
        }
    }

    #[test]
    fn serial_handler_can_publish_to_itself_without_deadlocking() {
        let log = within(|| {
            let log = Arc::new(Mutex::new(Vec::new()));
            let stream = EventStream::new(Arc::new(Executor::Serial));
            stream.attach(Countdown(Arc::clone(&log))).unwrap();
            stream.publish(Tick(2));
            log.lock().unwrap().clone()
        });

        // Each self-published Tick runs after the current handle returns,
        // never nested inside it, and all of them before publish returns.
        assert_eq!(
            log,
            vec![
                "enter 2", "exit 2", "enter 1", "exit 1", "enter 0", "exit 0"
            ]
        );
    }

    struct Serve(u32);
    struct Volley;
    impl Event for Serve {}
    impl Event for Volley {}

    struct PlayerA(Arc<AtomicUsize>);
    struct PlayerB;

    impl EventHandler for PlayerA {
        fn start(&mut self, ctx: &EventContext<'_, Self>) -> RadiateResult<()> {
            ctx.subscribe::<Serve>();
            ctx.subscribe::<Volley>();
            Ok(())
        }
    }

    impl Handler<Serve> for PlayerA {
        fn handle(&mut self, event: &Serve, ctx: &EventContext<'_, Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
            if event.0 > 0 {
                ctx.publish(Serve(event.0 - 1));
            }
        }
    }

    impl Handler<Volley> for PlayerA {
        fn handle(&mut self, _: &Volley, _: &EventContext<'_, Self>) {
            self.0.fetch_add(100, Ordering::SeqCst);
        }
    }

    impl EventHandler for PlayerB {
        fn start(&mut self, ctx: &EventContext<'_, Self>) -> RadiateResult<()> {
            ctx.subscribe::<Serve>();
            Ok(())
        }
    }

    impl Handler<Serve> for PlayerB {
        fn handle(&mut self, _: &Serve, ctx: &EventContext<'_, Self>) {
            ctx.publish(Volley);
        }
    }

    #[test]
    fn serial_cycle_between_two_handlers_does_not_deadlock() {
        // A handles Serve and publishes Serve; B handles Serve and publishes
        // Volley back to A. Both paths re-enter A while it is running.
        let hits = within(|| {
            let hits = Arc::new(AtomicUsize::new(0));
            let stream = EventStream::new(Arc::new(Executor::Serial));
            stream.attach(PlayerA(Arc::clone(&hits))).unwrap();
            stream.attach(PlayerB).unwrap();
            stream.publish(Serve(2));
            hits.load(Ordering::SeqCst)
        });

        // Three Serves (2, 1, 0) reach A, and B turns each into a Volley for A.
        assert_eq!(hits, 3 + 3 * 100);
    }

    struct Primed(Arc<AtomicUsize>);

    impl EventHandler for Primed {
        fn start(&mut self, ctx: &EventContext<'_, Self>) -> RadiateResult<()> {
            ctx.subscribe::<Tick>();
            ctx.publish(Tick(0));
            Ok(())
        }
    }

    impl Handler<Tick> for Primed {
        fn handle(&mut self, _: &Tick, _: &EventContext<'_, Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn start_can_publish_to_its_own_subscription() {
        let handled = within(|| {
            let handled = Arc::new(AtomicUsize::new(0));
            let stream = EventStream::new(Arc::new(Executor::Serial));
            stream.attach(Primed(Arc::clone(&handled))).unwrap();
            handled.load(Ordering::SeqCst)
        });

        assert_eq!(
            handled, 1,
            "the Tick published in start runs once start returns"
        );
    }

    struct Ping;
    impl Event for Ping {}

    /// Turns a deferred Ping into a Tick for itself, so flush must wait on a
    /// delivery that was queued behind the running handler.
    struct Echo(Arc<AtomicUsize>);

    impl EventHandler for Echo {
        fn start(&mut self, ctx: &EventContext<'_, Self>) -> RadiateResult<()> {
            ctx.subscribe::<DeferredEvent<Ping>>();
            ctx.subscribe::<Tick>();
            Ok(())
        }
    }

    impl Handler<DeferredEvent<Ping>> for Echo {
        fn handle(&mut self, _: &DeferredEvent<Ping>, ctx: &EventContext<'_, Self>) {
            ctx.publish(Tick(0));
        }
    }

    impl Handler<Tick> for Echo {
        fn handle(&mut self, _: &Tick, _: &EventContext<'_, Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn flush_waits_for_events_a_handler_queued_to_itself() {
        for executor in [Executor::Serial, Executor::FixedSizedWorkerPool(4)] {
            let handled = within(move || {
                let handled = Arc::new(AtomicUsize::new(0));
                let stream = EventStream::new(Arc::new(executor));
                stream.attach(Echo(Arc::clone(&handled))).unwrap();
                stream.defer(Ping);
                stream.flush();
                handled.load(Ordering::SeqCst)
            });

            assert_eq!(handled, 1);
        }
    }

    struct Even(usize);
    struct Odd(usize);
    struct Done;
    impl Event for Even {}
    impl Event for Odd {}
    impl Event for Done {}

    /// Records the sequence numbers it sees across two event types, and reports
    /// whether it was ever entered by two threads at once.
    struct Sequencer {
        seen: Vec<usize>,
        active: Arc<AtomicUsize>,
        overlapped: bool,
        report: mpsc::Sender<(Vec<usize>, bool)>,
    }

    impl Sequencer {
        fn record(&mut self, n: usize) {
            if self.active.fetch_add(1, Ordering::SeqCst) != 0 {
                self.overlapped = true;
            }
            self.seen.push(n);
            self.active.fetch_sub(1, Ordering::SeqCst);
        }
    }

    impl EventHandler for Sequencer {
        fn start(&mut self, ctx: &EventContext<'_, Self>) -> RadiateResult<()> {
            ctx.subscribe::<Even>();
            ctx.subscribe::<Odd>();
            ctx.subscribe::<Done>();
            Ok(())
        }
    }

    impl Handler<Even> for Sequencer {
        fn handle(&mut self, event: &Even, _: &EventContext<'_, Self>) {
            self.record(event.0);
        }
    }

    impl Handler<Odd> for Sequencer {
        fn handle(&mut self, event: &Odd, _: &EventContext<'_, Self>) {
            self.record(event.0);
        }
    }

    impl Handler<Done> for Sequencer {
        fn handle(&mut self, _: &Done, _: &EventContext<'_, Self>) {
            let seen = std::mem::take(&mut self.seen);
            self.report.send((seen, self.overlapped)).unwrap();
        }
    }

    #[test]
    fn pooled_handler_sees_events_of_every_type_in_publish_order() {
        const N: usize = 10_000;

        let stream = EventStream::new(Arc::new(Executor::FixedSizedWorkerPool(4)));
        let (report, results) = mpsc::channel();
        stream
            .attach(Sequencer {
                seen: Vec::new(),
                active: Arc::new(AtomicUsize::new(0)),
                overlapped: false,
                report,
            })
            .unwrap();

        for i in 0..N {
            if i % 2 == 0 {
                stream.publish(Even(i));
            } else {
                stream.publish(Odd(i));
            }
        }
        stream.publish(Done);

        let (seen, overlapped) = results.recv_timeout(GENEROUS).unwrap();
        assert!(!overlapped, "handler ran on two threads at once");
        assert_eq!(seen, (0..N).collect::<Vec<_>>());
    }
}
