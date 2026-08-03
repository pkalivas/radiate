//! Prototype: does `EngineEvent<T>` work as a message type on the new
//! `radiate_core::Bus` as-is, and can a handler reach `ThreadSync` from
//! inside `.handle()` to actually stop the run? Not wired into
//! `GeneticEngine` yet — this only proves the integration path.

use radiate_core::{Bus, EventHandler, Executor, Score, ThreadSync};
use radiate_engines::{EngineEvent, EngineEventInner};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

struct StopOnImprovement {
    handled: Arc<(Mutex<bool>, Condvar)>,
}

impl EventHandler<EngineEvent<i32>> for StopOnImprovement {
    fn handle(&mut self, event: EngineEvent<i32>) {
        if let EngineEventInner::Improvement(..) = event.inner() {
            event.sync().stop();
        }

        let (handled, cv) = &*self.handled;
        *handled.lock().unwrap() = true;
        cv.notify_all();
    }
}

#[test]
fn engine_event_flows_through_generic_bus_and_can_stop_from_handler() {
    let bus = Bus::new(Arc::new(Executor::default()));
    let handled = Arc::new((Mutex::new(false), Condvar::new()));

    bus.subscribe::<EngineEvent<i32>, _>(StopOnImprovement {
        handled: Arc::clone(&handled),
    });

    let sync = ThreadSync::new();
    assert!(!sync.is_stopped());

    let event = EngineEvent::new(
        sync.clone(),
        EngineEventInner::Improvement(3, 42, Score::from(1.0)),
    );
    bus.publish(event);

    let (lock, cv) = &*handled;
    let mut done = lock.lock().unwrap();
    while !*done {
        let (guard, timeout) = cv.wait_timeout(done, Duration::from_secs(2)).unwrap();
        done = guard;
        if timeout.timed_out() && !*done {
            panic!("handler never ran");
        }
    }

    assert!(sync.is_stopped(), "handler should have stopped the run");
}

#[test]
fn non_improvement_events_do_not_trigger_stop() {
    let bus = Bus::new(Arc::new(Executor::default()));
    let handled = Arc::new((Mutex::new(false), Condvar::new()));

    bus.subscribe::<EngineEvent<i32>, _>(StopOnImprovement {
        handled: Arc::clone(&handled),
    });

    let sync = ThreadSync::new();
    let event: EngineEvent<i32> = EngineEvent::new(sync.clone(), EngineEventInner::EpochStart(1));
    bus.publish(event);

    let (lock, cv) = &*handled;
    let mut done = lock.lock().unwrap();
    while !*done {
        let (guard, timeout) = cv.wait_timeout(done, Duration::from_secs(2)).unwrap();
        done = guard;
        if timeout.timed_out() && !*done {
            panic!("handler never ran");
        }
    }

    assert!(!sync.is_stopped());
}
