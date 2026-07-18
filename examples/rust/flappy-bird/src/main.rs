mod game;
mod render;
mod swarm;

use bevy::prelude::*;
use radiate::ops;
use radiate::prelude::*;
use render::{SnapshotReceiver, background_color, setup, sync_snapshot_to_entities};
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;
use swarm::{FlappySwarm, Snapshot};

const INPUT_SIZE: usize = 5;
const OUTPUT_SIZE: usize = 1;
const MAX_GENERATIONS: usize = 500;
const MAX_TRAIN_SECONDS: f64 = 3600.0;

fn main() {
    let (tx, rx) = mpsc::channel::<Snapshot>();

    thread::spawn(move || run_evolution(tx));

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
        .add_systems(Startup, setup)
        .add_systems(Update, sync_snapshot_to_entities)
        .run();
}

fn run_evolution(tx: mpsc::Sender<Snapshot>) {
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
        // generation with the *whole* population, so every bird in a
        // generation shares one clock and one set of pipes -- which is what
        // makes the live "swarm" view in the Bevy window meaningful. Adding
        // `.parallel()` would split the population into per-worker chunks,
        // each running its own disconnected simulation.
        .batch_fitness_fn(FlappySwarm::new(tx))
        .build();

    let _ = engine
        .iter()
        .logging()
        .until_seconds(MAX_TRAIN_SECONDS)
        .take(MAX_GENERATIONS)
        .last();
}
