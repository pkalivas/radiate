mod game;
mod render;
mod speed;
mod swarm;

use bevy::prelude::*;
use radiate::ops;
use radiate::prelude::*;
use render::{
    SimSpeedRes, SnapshotReceiver, background_color, handle_speed_input, setup,
    sync_snapshot_to_entities,
};
use speed::SimSpeed;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use swarm::{FlappySwarm, Snapshot};

const INPUT_SIZE: usize = 5;
const OUTPUT_SIZE: usize = 1;
const MAX_GENERATIONS: usize = 500;
const MAX_TRAIN_SECONDS: f64 = 3600.0;

fn main() {
    let (tx, rx) = mpsc::channel::<Snapshot>();
    let speed = SimSpeed::new();

    let worker_speed = speed.clone();
    thread::spawn(move || run_evolution(tx, worker_speed));

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Radiate — Flappy Bird NEAT".into(),
                resolution: (900_u32, 700_u32).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(background_color())
        .insert_resource(SnapshotReceiver(Mutex::new(rx)))
        .insert_resource(SimSpeedRes(speed))
        .add_systems(Startup, setup)
        .add_systems(Update, (sync_snapshot_to_entities, handle_speed_input))
        .run();
}

fn run_evolution(tx: mpsc::Sender<Snapshot>, speed: SimSpeed) {
    let store = vec![
        (NodeType::Input, Op::vars(0..INPUT_SIZE)),
        (NodeType::Edge, vec![Op::weight()]),
        (NodeType::Vertex, ops::activation_ops()),
        (NodeType::Output, vec![Op::sigmoid()]),
    ];

    let codec = GraphCodec::weighted_directed(INPUT_SIZE, OUTPUT_SIZE, store).with_max_nodes(20);

    let engine = GeneticEngine::builder()
        .codec(codec)
        .offspring_selector(TournamentSelector::new(4))
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.04, 0.05),
            GraphMutator::new(0.03, 0.03)
        ))
        // Intentionally no `.parallel()` here: the default `Executor::Serial`
        // guarantees `FlappySwarm::evaluate` is called exactly once per
        // generation with every member who _needs_ evaluation, so every bird in a
        // generation shares one clock and one set of pipes -- which is what
        // makes the live "swarm" view in the Bevy window meaningful. Adding
        // `.parallel()` would split the population into per-worker chunks,
        // each running its own disconnected simulation.
        .batch_fitness_fn(FlappySwarm::new(tx.clone(), speed.clone()))
        .build();

    let final_generation = engine
        .iter()
        .until_seconds(MAX_TRAIN_SECONDS)
        .take(MAX_GENERATIONS)
        .last();

    // `Generation::value()`/`score()` already track the best-ever genome
    // (the engine only updates them on improvement), so the last yielded generation's
    // value *is* the best one found across the whole run.
    let Ok(final_generation) = final_generation else {
        return;
    };
    let best_graph = final_generation.value().clone();
    let best_score = final_generation.score().as_f32();

    // Keep showing the winner off, looping through fresh courses, so the
    // window ends on a live demo.
    let mut seed = MAX_GENERATIONS as u64 + 1;
    loop {
        swarm::replay_best(&best_graph, best_score, &tx, &speed, seed);
        seed += 1;
        thread::sleep(Duration::from_millis(600));
    }
}
