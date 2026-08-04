use radiate_core::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// #[derive(Debug)]
// struct Counted(u64);

// fn wait_until<F: Fn() -> bool>(timeout: Duration, cond: F) -> bool {
//     let start = Instant::now();
//     while !cond() {
//         if start.elapsed() > timeout {
//             return false;
//         }
//         std::thread::yield_now();
//     }
//     true
// }

// fn parallel_broker(num_workers: usize) -> MessageBroker {
//     MessageBroker::new(Arc::new(Executor::FixedSizedWorkerPool(num_workers)))
// }

// fn serial_broker() -> MessageBroker {
//     MessageBroker::new(Arc::new(Executor::Serial))
// }

// #[test]
// fn single_actor_serial_executor_throughput() {
//     const N: u64 = 50_000;

//     let system = serial_broker();
//     let received = Arc::new(AtomicU64::new(0));
//     let received2 = Arc::clone(&received);

//     system
//         .on::<Counted>()
//         .handle(move |_msg: &Counted, _ctx: &EventContext| {
//             received2.fetch_add(1, Ordering::Relaxed);
//         });

//     let start = Instant::now();
//     for i in 0..N {
//         system.send(Counted(i));
//     }
//     let elapsed = start.elapsed();

//     let throughput = N as f64 / elapsed.as_secs_f64();

//     println!("[serial]     {N} messages in {elapsed:?} ({throughput:.0} msgs/sec)");
//     assert_eq!(received.load(Ordering::Relaxed), N);
// }

// #[test]
// fn single_actor_parallel_executor_throughput_preserves_order() {
//     const N: u64 = 500_000;

//     let system = parallel_broker(4);
//     let received = Arc::new(AtomicU64::new(0));
//     let order = Arc::new(Mutex::new(Vec::with_capacity(N as usize)));

//     let received2 = Arc::clone(&received);
//     let order2 = Arc::clone(&order);
//     system
//         .on::<Counted>()
//         .handle(move |msg: &Counted, _ctx: &EventContext| {
//             order2.lock().unwrap().push(msg.0);
//             received2.fetch_add(1, Ordering::Relaxed);
//         });

//     let start = Instant::now();
//     for i in 0..N {
//         system.send(Counted(i));
//     }
//     let ok = wait_until(Duration::from_secs(20), || {
//         received.load(Ordering::Relaxed) == N
//     });
//     let elapsed = start.elapsed();

//     assert!(
//         ok,
//         "timed out waiting for {N} messages, saw {}",
//         received.load(Ordering::Relaxed)
//     );

//     // A single actor's mailbox drains single-flight (the `scheduled` CAS in
//     // Actor::tell/drain) regardless of how many worker threads the executor
//     // has, so per-actor FIFO delivery must hold even under a 4-worker pool.
//     let seen = order.lock().unwrap();
//     let expected: Vec<u64> = (0..N).collect();
//     assert_eq!(
//         *seen, expected,
//         "per-actor ordering violated under parallel executor"
//     );

//     let throughput = N as f64 / elapsed.as_secs_f64();
//     println!("[parallel]   {N} messages in {elapsed:?} ({throughput:.0} msgs/sec)");
// }

// #[test]
// fn fan_out_to_many_subscribers_throughput() {
//     const N: u64 = 20_000;
//     const SUBSCRIBERS: usize = 50;

//     let system = parallel_broker(4);
//     let total_received = Arc::new(AtomicU64::new(0));

//     for _ in 0..SUBSCRIBERS {
//         let total = Arc::clone(&total_received);
//         system
//             .on::<Counted>()
//             .handle(move |_msg: &Counted, _ctx: &EventContext| {
//                 total.fetch_add(1, Ordering::Relaxed);
//             });
//     }

//     let target = N * SUBSCRIBERS as u64;

//     let start = Instant::now();
//     for i in 0..N {
//         system.send(Counted(i));
//     }
//     let ok = wait_until(Duration::from_secs(20), || {
//         total_received.load(Ordering::Relaxed) == target
//     });
//     let elapsed = start.elapsed();

//     assert!(
//         ok,
//         "timed out: expected {target} total deliveries, saw {}",
//         total_received.load(Ordering::Relaxed)
//     );

//     let throughput = target as f64 / elapsed.as_secs_f64();
//     println!(
//         "[fan-out]    {N} messages x {SUBSCRIBERS} subscribers = {target} deliveries in {elapsed:?} ({throughput:.0} deliveries/sec)"
//     );
// }

// #[test]
// fn concurrent_producers_lose_no_messages() {
//     const PRODUCERS: u64 = 8;
//     const PER_PRODUCER: u64 = 10_000;
//     const N: u64 = PRODUCERS * PER_PRODUCER;

//     let system = parallel_broker(4);
//     let received = Arc::new(AtomicU64::new(0));
//     let received2 = Arc::clone(&received);

//     system
//         .on::<Counted>()
//         .handle(move |_msg: &Counted, _ctx: &EventContext| {
//             received2.fetch_add(1, Ordering::Relaxed);
//         });

//     let start = Instant::now();
//     std::thread::scope(|scope| {
//         for _ in 0..PRODUCERS {
//             let system = system.clone();
//             scope.spawn(move || {
//                 for i in 0..PER_PRODUCER {
//                     system.send(Counted(i));
//                 }
//             });
//         }
//     });
//     let ok = wait_until(Duration::from_secs(20), || {
//         received.load(Ordering::Relaxed) == N
//     });
//     let elapsed = start.elapsed();

//     assert!(
//         ok,
//         "timed out: {PRODUCERS} concurrent producers, expected {N}, saw {}",
//         received.load(Ordering::Relaxed)
//     );
//     assert_eq!(
//         received.load(Ordering::Relaxed),
//         N,
//         "messages lost or duplicated under concurrent producers"
//     );

//     let throughput = N as f64 / elapsed.as_secs_f64();
//     println!(
//         "[concurrent] {PRODUCERS} producers x {PER_PRODUCER} = {N} messages in {elapsed:?} ({throughput:.0} msgs/sec)"
//     );
// }
