use crate::game::{self, World};
use crate::speed::SimSpeed;
use radiate::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::thread;

/// A single tick's worth of render-relevant state, streamed from the worker
/// thread (which owns the `GeneticEngine`) to the Bevy app on the main thread.
pub struct Snapshot {
    pub generation: usize,
    pub tick: u32,
    pub birds: Vec<(f32, bool)>,
    pub pipes: Vec<(f32, f32, f32)>,
    /// `Some(score)` marks this as a post-training solo replay of the best
    /// genome found, rather than a live training-generation frame.
    pub best_score: Option<f32>,
}

/// Evaluates an entire generation's population in one shared simulation —
/// all birds fly against the same pipes on the same clock, which is what
/// lets the renderer show a live "swarm" rather than N independent replays.
///
/// This only works because the engine is built *without* `.parallel()`, so
/// `Executor::Serial` hands the whole population to `evaluate()` in a single
/// call instead of splitting it into per-worker chunks. See main.rs.
pub struct FlappySwarm {
    tx: Sender<Snapshot>,
    speed: SimSpeed,
    generation: AtomicUsize,
}

impl FlappySwarm {
    pub fn new(tx: Sender<Snapshot>, speed: SimSpeed) -> Self {
        FlappySwarm {
            tx,
            speed,
            generation: AtomicUsize::new(0),
        }
    }
}

impl BatchFitnessFunction<Graph<Op<f32>>, f32> for FlappySwarm {
    fn evaluate(&self, graphs: Vec<Graph<Op<f32>>>) -> Vec<f32> {
        let generation = self.generation.fetch_add(1, Ordering::Relaxed);
        let mut world = World::new(graphs.len(), generation as u64 + 1);

        loop {
            let flaps: Vec<bool> = graphs
                .iter()
                .enumerate()
                .map(|(i, graph)| {
                    if !world.birds[i].alive {
                        return false;
                    }
                    let inputs = world.bird_inputs(i);
                    let outputs = graph.eval(&vec![inputs.to_vec()]);
                    outputs[0][0] > 0.5
                })
                .collect();

            world.step(&flaps);

            let snapshot = Snapshot {
                generation,
                tick: world.tick,
                birds: world.birds.iter().map(|b| (b.y, b.alive)).collect(),
                pipes: world
                    .pipes
                    .iter()
                    .map(|p| (p.x, p.gap_top, p.gap_bottom))
                    .collect(),
                best_score: None,
            };
            // Rendering is best-effort: if the window has closed, the
            // receiver is gone and send() fails — training keeps going
            // headless rather than panicking.
            let _ = self.tx.send(snapshot);

            let delay = self.speed.delay();
            if !delay.is_zero() {
                thread::sleep(delay);
            }

            if world.all_dead() || world.tick >= game::MAX_TICKS {
                break;
            }
        }

        world.birds.iter().map(|b| b.fitness()).collect()
    }
}

/// Flies the best-ever genome through a single fresh course, streaming
/// `Snapshot`s tagged with `best_score` so the renderer can show it as the
/// final showcase rather than another training generation.
pub fn replay_best(graph: &Graph<Op<f32>>, best_score: f32, tx: &Sender<Snapshot>, speed: &SimSpeed, seed: u64) {
    let mut world = World::new(1, seed);

    loop {
        let inputs = world.bird_inputs(0);
        let outputs = graph.eval(&vec![inputs.to_vec()]);
        let flap = outputs[0][0] > 0.5;
        world.step(&[flap]);

        let snapshot = Snapshot {
            generation: 0,
            tick: world.tick,
            birds: world.birds.iter().map(|b| (b.y, b.alive)).collect(),
            pipes: world
                .pipes
                .iter()
                .map(|p| (p.x, p.gap_top, p.gap_bottom))
                .collect(),
            best_score: Some(best_score),
        };
        let _ = tx.send(snapshot);

        if world.all_dead() || world.tick >= game::MAX_TICKS {
            break;
        }

        let delay = speed.delay();
        if !delay.is_zero() {
            thread::sleep(delay);
        }
    }
}
