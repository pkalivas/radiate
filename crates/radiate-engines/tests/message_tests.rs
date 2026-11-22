use radiate_core::{Executor, error::RadiateResult};
use radiate_engines::events::{Event, EventContext, EventHandler, EventStream, On};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// Long enough that a passing run never hits it; only a hang or regression does.
const GENEROUS: Duration = Duration::from_secs(5);

struct Ping;
#[allow(dead_code)]
struct Pong;
struct Add(usize);
struct Tick(u32);

impl Event for Ping {}
impl Event for Pong {}
impl Event for Add {}
impl Event for Tick {}

fn serial() -> EventStream {
    EventStream::from(Executor::Inline)
}

fn pool() -> EventStream {
    EventStream::from(Executor::new_thread_pool(4))
}

/// Subscribe a closure that counts every `E` it sees.
fn count<E: Event>(stream: &EventStream) -> Arc<AtomicUsize> {
    let seen = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&seen);
    stream.subscribe(move |_: &E| {
        counter.fetch_add(1, Ordering::SeqCst);
    });
    seen
}

/// Run `f` on its own thread and fail (instead of hanging the suite) if it
/// doesn't finish. Used for paths that have deadlocked before.
fn within<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || tx.send(f()).unwrap());
    rx.recv_timeout(GENEROUS)
        .expect("timed out: likely deadlocked")
}

mod publish {
    use super::*;

    /// On a pool, events queue behind a busy handler. Unsubscribing has to
    /// stop those too, not just future publishes: a handler that has gone
    /// away (like a closed UI) must not be called again.
    #[test]
    fn unsubscribe_drops_deliveries_already_queued() {
        let stream = pool();
        let handled = Arc::new(AtomicUsize::new(0));
        let (started_tx, started) = mpsc::channel();
        let (release, release_rx) = mpsc::channel::<()>();
        let release_rx = Mutex::new(release_rx);

        let counter = Arc::clone(&handled);
        let subscription = stream.subscribe(move |_: &Ping| {
            if counter.fetch_add(1, Ordering::SeqCst) == 0 {
                started_tx.send(()).unwrap();
                release_rx.lock().unwrap().recv_timeout(GENEROUS).unwrap();
            }
        });

        stream.publish(Ping).unwrap();
        started.recv_timeout(GENEROUS).unwrap();

        // The handler is busy with the first, so these four queue.
        for _ in 0..4 {
            stream.publish(Ping).unwrap();
        }

        subscription.unsubscribe();
        release.send(()).unwrap();
        stream.drain().unwrap();

        assert_eq!(handled.load(Ordering::SeqCst), 1);
    }

    struct Summer(Arc<AtomicUsize>);

    impl On<Add> for Summer {
        fn handle(&mut self, event: &Add, _: &EventContext<'_, Self>) {
            self.0.fetch_add(event.0, Ordering::SeqCst);
        }
    }

    #[test]
    fn every_subscriber_sees_every_event_whether_handler_or_closure() {
        let stream = serial();
        let by_handler = Arc::new(AtomicUsize::new(0));
        let by_closure = Arc::new(AtomicUsize::new(0));

        stream.subscribe(Summer(Arc::clone(&by_handler)));
        stream.subscribe({
            let total = Arc::clone(&by_closure);
            move |event: &Add| {
                total.fetch_add(event.0, Ordering::SeqCst);
            }
        });

        stream.publish(Add(3)).unwrap();
        stream.publish(Add(4)).unwrap();

        assert_eq!(by_handler.load(Ordering::SeqCst), 7);
        assert_eq!(by_closure.load(Ordering::SeqCst), 7);
    }

    #[test]
    fn unsubscribing_stops_further_delivery() {
        let stream = serial();
        let (first, second) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));

        let by_handle = stream.subscribe({
            let seen = Arc::clone(&first);
            move |_: &Ping| {
                seen.fetch_add(1, Ordering::SeqCst);
            }
        });
        let by_id = stream.subscribe({
            let seen = Arc::clone(&second);
            move |_: &Ping| {
                seen.fetch_add(1, Ordering::SeqCst);
            }
        });

        stream.publish(Ping).unwrap();
        by_handle.unsubscribe();
        stream.unsubscribe(by_id.id());
        stream.publish(Ping).unwrap();

        assert_eq!(first.load(Ordering::SeqCst), 1);
        assert_eq!(second.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn lazy_publish_skips_construction_with_no_subscribers() {
        let stream = serial();
        let built = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&built);

        stream
            .lazy_publish(move || {
                counter.fetch_add(1, Ordering::SeqCst);
                Ping
            })
            .unwrap();

        assert_eq!(built.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn subscribers_on_the_same_event_keep_independent_schedules() {
        let stream = serial();
        let every_two = stream.subscribe(|_: &Ping| {});
        let every_three = stream.subscribe(|_: &Ping| {});
        every_two.schedule(2_usize).unwrap();
        every_three.schedule(3_usize).unwrap();

        for _ in 0..6 {
            stream.lazy_publish(|| Ping).unwrap();
        }

        // An earlier subscriber being due must not stop later ones from ticking.
        let two = every_two.stats();
        let three = every_three.stats();
        assert_eq!((two.delivered, two.handled, two.skipped), (3, 3, 3));
        assert_eq!((three.delivered, three.handled, three.skipped), (2, 2, 4));
    }

    #[test]
    fn stream_stats_report_live_subscriptions_by_name_and_event() {
        let stream = serial();
        let pings = stream.subscribe(|_: &Ping| {}).with_name("pings");
        let adds = stream.subscribe(|_: &Add| {});

        stream.publish(Ping).unwrap();
        stream.publish(Add(1)).unwrap();
        stream.publish(Add(2)).unwrap();

        let report = stream.snapshot();
        assert_eq!(report.subscribers.len(), 2);

        let ping = report
            .subscribers
            .iter()
            .find(|s| s.id == pings.id())
            .unwrap();
        assert_eq!(ping.name.as_ref().map(|n| n.as_str()), Some("pings"));
        assert!(ping.event.ends_with("Ping"));
        assert_eq!((ping.stats.delivered, ping.stats.handled), (1, 1));
        assert_eq!(ping.stats.queued(), 0);

        let add = report
            .subscribers
            .iter()
            .find(|s| s.id == adds.id())
            .unwrap();
        assert_eq!(add.name, None);
        assert_eq!((add.stats.delivered, add.stats.handled), (2, 2));

        adds.unsubscribe();
        let report = stream.snapshot();
        assert_eq!(report.subscribers.len(), 1);
        assert_eq!(report.subscribers[0].id, pings.id());
    }

    #[test]
    fn stream_stats_count_publishes_per_event_type() {
        let stream = serial();
        let pings = stream.subscribe(|_: &Ping| {});
        pings.schedule(2_usize).unwrap();
        stream.subscribe(|_: &Add| {}).schedule(3_usize).unwrap();

        // Eager: built every time, delivered to nobody when the schedule skips.
        for _ in 0..4 {
            stream.publish(Ping).unwrap();
        }
        // Lazy: only built when the schedule is due.
        for _ in 0..3 {
            stream.lazy_publish(|| Add(1)).unwrap();
        }

        let event = |report: &radiate_engines::StreamSnapshot, name: &str| {
            let stats = report.events.iter().find(|e| e.event.ends_with(name));
            let stats = stats.unwrap_or_else(|| panic!("no stats for {name}"));
            (stats.published, stats.unbuilt, stats.undelivered)
        };

        let first = stream.snapshot();
        assert_eq!(event(&first, "Ping"), (4, 0, 2));
        assert_eq!(event(&first, "Add"), (1, 2, 0));
        assert_eq!(first.event_count, 5);
        assert_eq!(stream.event_count(), 5);

        // A type keeps its counts after its last subscription goes.
        stream.unsubscribe(pings.id());
        stream.publish(Ping).unwrap();

        let second = stream.snapshot();
        assert_eq!(event(&second, "Ping"), (5, 0, 3));
    }

    #[test]
    fn a_panicking_handler_poisons_the_lock_instead_of_being_isolated() {
        let stream = serial();
        stream.subscribe(|_: &Add| panic!("simulated failure"));

        let publish =
            || std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| stream.publish(Add(1))));

        assert!(publish().is_err());
        // Every later delivery to that subscriber panics too: a panicking
        // handler should stop the run, not be quietly walled off.
        assert!(publish().is_err());
    }

    #[test]
    fn a_pooled_executor_runs_handlers_off_the_publisher_thread() {
        let stream = pool();
        let (tx, rx) = mpsc::channel();
        stream.subscribe(move |_: &Ping| {
            tx.send(std::thread::current().id()).unwrap();
        });

        stream.publish(Ping).unwrap();

        let handler_thread = rx.recv_timeout(GENEROUS).unwrap();
        assert_ne!(handler_thread, std::thread::current().id());
    }
}

mod stats {
    use super::*;

    #[test]
    fn serial_deliveries_run_inline_and_are_timed() {
        let stream = serial();
        let subscription = stream.subscribe(|_: &Ping| {});

        for _ in 0..5 {
            stream.publish(Ping).unwrap();
        }
        stream.drain().unwrap();

        let stats = subscription.stats();
        assert_eq!((stats.delivered, stats.handled, stats.inline), (5, 5, 5));
        assert!(stats.mean_wait().is_some() && stats.mean_handling().is_some());
        assert_eq!(stats.queued(), 0);
    }

    #[test]
    fn pooled_deliveries_are_queued_and_timed() {
        let stream = pool();
        let subscription = stream.subscribe(|_: &Ping| {});

        for _ in 0..50 {
            stream.publish(Ping).unwrap();
        }
        stream.drain().unwrap();

        let stats = subscription.stats();
        assert_eq!((stats.delivered, stats.handled, stats.inline), (50, 50, 0));
        assert!(stats.mean_wait().is_some() && stats.mean_handling().is_some());
        assert_eq!(stats.queued(), 0);
    }

    #[test]
    fn panics_and_refused_deliveries_are_accounted_for() {
        let stream = serial();
        let subscription = stream.subscribe(|_: &Add| panic!("simulated failure"));
        let publish =
            || std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| stream.publish(Add(1))));

        assert!(publish().is_err());
        // The handler is poisoned now, so this one never reaches it.
        assert!(publish().is_err());

        let stats = subscription.stats();
        assert_eq!(
            (stats.delivered, stats.panicked, stats.discarded),
            (2, 1, 1)
        );
        assert_eq!(stats.handled, 0);
        assert_eq!(
            stats.handling,
            Duration::ZERO,
            "only finished calls are timed"
        );
        assert_eq!(stats.queued(), 0);
    }
}

mod ordering {
    use super::*;

    /// Counts down by publishing a smaller Tick to itself, recording when each
    /// handle call starts and ends.
    struct Countdown(Arc<Mutex<Vec<String>>>);

    impl EventHandler for Countdown {
        fn start(&mut self, ctx: &EventContext<'_, Self>) -> RadiateResult<()> {
            ctx.subscribe::<Tick>();
            Ok(())
        }
    }

    impl On<Tick> for Countdown {
        fn handle(&mut self, event: &Tick, ctx: &EventContext<'_, Self>) {
            self.0.lock().unwrap().push(format!("enter {}", event.0));
            if event.0 > 0 {
                ctx.publish(Tick(event.0 - 1)).unwrap();
            }
            self.0.lock().unwrap().push(format!("exit {}", event.0));
        }
    }

    #[test]
    fn serial_handler_can_publish_to_itself_without_deadlocking() {
        let log = within(|| {
            let log = Arc::new(Mutex::new(Vec::new()));
            let stream = serial();
            stream.attach(Countdown(Arc::clone(&log))).unwrap();
            stream.publish(Tick(2)).unwrap();
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

    impl On<Serve> for PlayerA {
        fn handle(&mut self, event: &Serve, ctx: &EventContext<'_, Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
            if event.0 > 0 {
                ctx.publish(Serve(event.0 - 1)).unwrap();
            }
        }
    }

    impl On<Volley> for PlayerA {
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

    impl On<Serve> for PlayerB {
        fn handle(&mut self, _: &Serve, ctx: &EventContext<'_, Self>) {
            ctx.publish(Volley).unwrap();
        }
    }

    #[test]
    fn serial_cycle_between_two_handlers_does_not_deadlock() {
        // A handles Serve and publishes Serve; B handles Serve and publishes
        // Volley back to A. Both paths re-enter A while it is running.
        let hits = within(|| {
            let hits = Arc::new(AtomicUsize::new(0));
            let stream = serial();
            stream.attach(PlayerA(Arc::clone(&hits))).unwrap();
            stream.attach(PlayerB).unwrap();
            stream.publish(Serve(2)).unwrap();
            hits.load(Ordering::SeqCst)
        });

        // Three Serves (2, 1, 0) reach A, and B turns each into a Volley for A.
        assert_eq!(hits, 3 + 3 * 100);
    }

    struct Primed(Arc<AtomicUsize>);

    impl EventHandler for Primed {
        fn start(&mut self, ctx: &EventContext<'_, Self>) -> RadiateResult<()> {
            ctx.subscribe::<Tick>();
            ctx.publish(Tick(0)).unwrap();
            Ok(())
        }
    }

    impl On<Tick> for Primed {
        fn handle(&mut self, _: &Tick, _: &EventContext<'_, Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn start_can_publish_to_its_own_subscription() {
        let handled = within(|| {
            let handled = Arc::new(AtomicUsize::new(0));
            serial().attach(Primed(Arc::clone(&handled))).unwrap();
            handled.load(Ordering::SeqCst)
        });

        assert_eq!(
            handled, 1,
            "the Tick published in start runs once start returns"
        );
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

    impl On<Even> for Sequencer {
        fn handle(&mut self, event: &Even, _: &EventContext<'_, Self>) {
            self.record(event.0);
        }
    }

    impl On<Odd> for Sequencer {
        fn handle(&mut self, event: &Odd, _: &EventContext<'_, Self>) {
            self.record(event.0);
        }
    }

    impl On<Done> for Sequencer {
        fn handle(&mut self, _: &Done, _: &EventContext<'_, Self>) {
            let seen = std::mem::take(&mut self.seen);
            self.report.send((seen, self.overlapped)).unwrap();
        }
    }

    #[test]
    fn pooled_handler_sees_events_of_every_type_in_publish_order() {
        const N: usize = 10_000;

        let stream = pool();
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
                stream.publish(Even(i)).unwrap();
            } else {
                stream.publish(Odd(i)).unwrap();
            }
        }
        stream.publish(Done).unwrap();

        let (seen, overlapped) = results.recv_timeout(GENEROUS).unwrap();
        assert!(!overlapped, "handler ran on two threads at once");
        assert_eq!(seen, (0..N).collect::<Vec<_>>());
    }
}

/// Throughput smoke tests. Run with:
///   cargo test -p radiate-engines --release --test message_tests -- --ignored --nocapture --test-threads=1
///
/// One at a time: run in parallel they compete for cores and the numbers drop by half.
mod throughput {
    use super::*;

    /// Publish `n` Ticks to `subscribers` counting subscribers (after a short
    /// warmup), drain, and print deliveries per second.
    fn measure(label: &str, stream: EventStream, subscribers: usize, n: usize) {
        let counts = (0..subscribers)
            .map(|_| count::<Tick>(&stream))
            .collect::<Vec<_>>();

        let run = |events: usize| {
            for i in 0..events {
                stream.publish(Tick(i as u32)).unwrap();
            }
            stream.drain().unwrap();
        };

        let warmup = n / 10;
        run(warmup);

        let start = Instant::now();
        run(n);
        let elapsed = start.elapsed();

        for seen in &counts {
            assert_eq!(seen.load(Ordering::SeqCst), warmup + n);
        }

        let deliveries = n * subscribers;
        println!(
            "[{label}] {deliveries} deliveries in {elapsed:?} = {:.0}/s",
            deliveries as f64 / elapsed.as_secs_f64()
        );
    }

    //     cargo test --release -p radiate-engines --test message_tests -- \
    //   --ignored --nocapture --test-threads=1 --exact \
    //   throughput::pool_single_subscriber \
    //   pool_tests::pool_logger_to_adder \
    //   throughput::serial_single_subscriber

    #[test]
    #[ignore = "throughput smoke test"]
    // [serial x1] 500000 deliveries in 257.693542ms = 1,940,289/s
    // serial x1] 500000 deliveries in 26.705292ms = 18,722,881/s
    fn serial_single_subscriber() {
        measure("serial x1", serial(), 1, 500_000);
    }

    #[test]
    #[ignore = "throughput smoke test"]
    // [serial x8] 800000 deliveries in 169.310792ms = 4,725,038/s
    fn serial_fanout() {
        measure("serial x8", serial(), 8, 100_000);
    }

    #[test]
    #[ignore = "throughput smoke test"]
    // [pool x1] 200000 deliveries in 252.708625ms = 791,425/s
    // [pool x1] 200000 deliveries in 31.361083ms = 6,377,331/s
    fn pool_single_subscriber() {
        measure("pool x1", pool(), 1, 200_000);
    }
}

#[cfg(test)]
mod pool_tests {
    use std::sync::atomic::AtomicI32;

    use super::*;

    #[derive(Debug)]
    struct Adder {
        total_count: Arc<AtomicI32>,
        relays: Arc<AtomicI32>,
        from_logger: Arc<AtomicI32>,
        handled: Arc<AtomicI32>,
    }

    #[derive(Debug)]
    struct Logger {
        relay_count: Arc<AtomicI32>,
        handled: Arc<AtomicI32>,
    }

    #[derive(Debug, Clone, Copy)]
    struct Tick(i32);
    impl Event for Tick {}

    #[derive(Debug, Clone, Copy)]
    struct Relay<T>(pub T);
    impl<T> Event for Relay<T> where T: Send + Sync + 'static {}

    #[derive(Debug, Clone, Copy)]
    struct FromLogger(i32);
    impl Event for FromLogger {}

    impl EventHandler for Adder {
        fn start(&mut self, ctx: &EventContext<'_, Self>) -> RadiateResult<()> {
            ctx.subscribe::<Tick>();
            ctx.subscribe::<Relay<i32>>();
            ctx.subscribe::<FromLogger>();

            Ok(())
        }
    }

    impl On<Tick> for Adder {
        fn handle(&mut self, event: &Tick, ctx: &EventContext<'_, Self>) {
            self.total_count.fetch_add(event.0, Ordering::SeqCst);
            self.handled.fetch_add(1, Ordering::SeqCst);
            ctx.publish(Relay(event.0)).unwrap();
        }
    }

    impl On<Relay<i32>> for Adder {
        fn handle(&mut self, event: &Relay<i32>, _: &EventContext<'_, Self>) {
            self.relays.fetch_add(event.0, Ordering::SeqCst);
            self.handled.fetch_add(1, Ordering::SeqCst);
        }
    }

    impl On<FromLogger> for Adder {
        fn handle(&mut self, event: &FromLogger, _ctx: &EventContext<'_, Self>) {
            self.from_logger.fetch_add(event.0, Ordering::SeqCst);
            self.handled.fetch_add(1, Ordering::SeqCst);
        }
    }

    impl EventHandler for Logger {
        fn start(&mut self, ctx: &EventContext<'_, Self>) -> RadiateResult<()> {
            ctx.subscribe::<Relay<i32>>();
            Ok(())
        }
    }

    impl On<Relay<i32>> for Logger {
        fn handle(&mut self, event: &Relay<i32>, ctx: &EventContext<'_, Self>) {
            self.relay_count.fetch_add(event.0, Ordering::SeqCst);
            self.handled.fetch_add(1, Ordering::SeqCst);
            ctx.publish(FromLogger(event.0)).unwrap();
        }
    }

    #[test]
    #[ignore = "throughput smoke test"]
    // [pool_logger_to_adder] 40000 deliveries in 28.829042ms = 1,387,490/s
    // [pool_logger_to_adder] 40000 deliveries in 28.807167ms = 1,388,543/s
    // [pool_logger_to_adder] 120000 deliveries in 38.944333ms = 3,081,321/s
    // [pool_logger_to_adder] 120000 deliveries in 14.414625ms = 8,324,878/s
    // 2,792,282
    fn pool_logger_to_adder() {
        const N: usize = 10_000;

        let total_count = Arc::new(AtomicI32::new(0));
        let relays = Arc::new(AtomicI32::new(0));
        let from_logger = Arc::new(AtomicI32::new(0));
        let relay_count = Arc::new(AtomicI32::new(0));
        let adder_handled = Arc::new(AtomicI32::new(0));
        let logger_handled = Arc::new(AtomicI32::new(0));
        let adder_two_handled = Arc::new(AtomicI32::new(0));

        let adder = Adder {
            total_count: total_count.clone(),
            relays: relays.clone(),
            from_logger: from_logger.clone(),
            handled: adder_handled.clone(),
        };

        let adder_two = Adder {
            total_count: total_count.clone(),
            relays: relays.clone(),
            from_logger: from_logger.clone(),
            handled: adder_two_handled.clone(),
        };

        let logger = Logger {
            relay_count: relay_count.clone(),
            handled: logger_handled.clone(),
        };

        let stream = EventStream::new(Executor::new_thread_pool(4));
        stream.attach(adder).unwrap();
        stream.attach(adder_two).unwrap();
        stream.attach(logger).unwrap();

        let run = |events: usize| {
            for i in 0..events {
                stream.publish(Tick(i as i32)).unwrap();
            }

            stream.close().unwrap();
        };

        let start = Instant::now();
        run(N);
        let elapsed = start.elapsed();

        let deliveries = stream
            .snapshot()
            .subscribers
            .iter()
            .map(|s| s.stats.delivered)
            .sum::<u64>();

        let label = "pool_logger_to_adder";
        let stream_metrics = stream.snapshot().stream_metrics();

        println!(
            "[{label}] {deliveries} deliveries in {elapsed:?} = {:.0}/s",
            deliveries as f64 / elapsed.as_secs_f64()
        );

        println!("{:#?}", stream_metrics);
        println!("{:#?}", stream.snapshot().utilization());
        println!(
            "Throughput: {:.0}/s",
            stream.snapshot().throughput().unwrap()
        );
    }
}
