#[cfg(test)]
mod event_stream_tests {
    use radiate_core::Executor;
    use radiate_engines::events::{
        EventContext, EventHandler, EventStream, Subscriber, Subscribes,
    };
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn serial() -> Arc<Executor> {
        Arc::new(Executor::Serial)
    }

    struct Add(usize);

    struct Counter {
        total: Arc<AtomicUsize>,
    }

    impl EventHandler<Add> for Counter {
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

    struct Doubler;
    impl EventHandler<Add> for Doubler {
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
    #[allow(dead_code)]
    struct Pong(u32);

    struct Both {
        pings: Arc<AtomicUsize>,
        pongs: Arc<AtomicUsize>,
    }

    impl Subscribes for Both {
        fn subscribe(subscriber: &Subscriber<Self>) {
            subscriber.subscribe::<Ping>();
            subscriber.subscribe::<Pong>();
        }
    }

    impl EventHandler<Ping> for Both {
        fn handle(&mut self, _: &Ping, _ctx: &EventContext<'_, Self>) {
            self.pings.fetch_add(1, Ordering::SeqCst);
        }
    }

    impl EventHandler<Pong> for Both {
        fn handle(&mut self, _: &Pong, _ctx: &EventContext<'_, Self>) {
            self.pongs.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn subscribes_trait_wires_up_more_than_one_event_type() {
        let stream = EventStream::new(serial());
        let pings = Arc::new(AtomicUsize::new(0));
        let pongs = Arc::new(AtomicUsize::new(0));

        stream.spawn_and_subscribe(Both {
            pings: Arc::clone(&pings),
            pongs: Arc::clone(&pongs),
        });

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

        stream.lazy_publish(move || {
            built_clone.fetch_add(1, Ordering::SeqCst);
            Ping(1)
        });

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
            .schedule(3_usize);

        for i in 0..6 {
            stream.lazy_publish(move || Ping(i));
        }

        // Due on the 3rd and 6th publish only.
        assert_eq!(total.load(Ordering::SeqCst), 2);
    }

    // --- the explicit design choice: panics propagate, they aren't isolated ---

    #[test]
    fn a_panicking_handler_poisons_the_lock_instead_of_being_isolated() {
        struct Boom;
        impl EventHandler<Add> for Boom {
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
