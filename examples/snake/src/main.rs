mod game;
mod snake;

use game::SnakeGame;
use radiate::prelude::*;
use snake::SnakeAI;
use std::collections::HashSet;
use std::io::{self, Write};
use std::sync::LazyLock;
use std::thread;
use std::time::Duration;

const MAX_GENERATIONS: usize = 500;
const MAX_SECONDS: f64 = 20.0;

const HEIGHT: i32 = 20;
const WIDTH: i32 = 30;
const MAX_STEPS: usize = 1500;

// Input state = 13, output actions = 4 (Up / Right / Down / Left)
const INPUT_SIZE: usize = 13;
const OUTPUT_SIZE: usize = 4;

const NUM_GAMES: usize = 1;
const GAMMA: f32 = 0.99;
const K_POTENTIAL: f32 = 2.0;
const STEP_PENALTY: f32 = -0.02;
const JITTER_PENALTY: f32 = -0.05;
const LOOP_WINDOW: usize = 12;
const LOOP_PENALTY: f32 = -0.15;
const EXPLORATION_BONUS: f32 = 0.2;
const EAT_REWARD: f32 = 200.0;
const SCORE_TERMINAL: f32 = 400.0;
const EARLY_DEATH_PENALTY: f32 = -120.0;
const MIN_SURVIVAL_STEPS: usize = 50;

const FOOD_POSITIONS: LazyLock<[(i32, i32); 1000]> = LazyLock::new(|| {
    let mut positions = [(0, 0); 1000];
    let mut idx = 0;

    while idx < 1000 {
        positions[idx] = (
            random_provider::range(0..WIDTH),
            random_provider::range(0..HEIGHT),
        );
        idx += 1;
    }

    positions
});

fn main() {
    let mut args = std::env::args();
    args.next();
    let engine_type = args.next().unwrap_or_else(|| "g".to_string());

    match engine_type.as_str() {
        "g" => run_graph_engine(),
        "n" => run_neural_net_engine(),
        _ => {
            eprintln!("Unknown engine type: {}", engine_type);
            std::process::exit(1);
        }
    }
}

fn run_graph_engine() {
    let store = vec![
        (NodeType::Input, Op::vars(0..INPUT_SIZE)),
        (NodeType::Edge, vec![Op::weight()]),
        (NodeType::Vertex, ops::activation_ops()),
        (NodeType::Output, vec![Op::sigmoid()]),
    ];

    let codec = GraphCodec::weighted_directed(INPUT_SIZE, OUTPUT_SIZE, store).with_max_nodes(10);

    let engine = GeneticEngine::builder()
        .codec(codec)
        .parallel()
        .offspring_selector(TournamentSelector::new(4))
        .alter(alters!(
            GraphCrossover::new(0.5, 0.5),
            OperationMutator::new(0.04, 0.05),
            GraphMutator::new(0.08, 0.04)
        ))
        .fitness_fn(|g: Graph<Op<f32>>| snake_fitness(SnakeAI::Graph(&g)))
        .build();

    // radiate::ui(engine)
    engine
        .iter()
        .logging()
        .limit(vec![
            Limit::Generation(MAX_GENERATIONS),
            Limit::Seconds(Duration::from_secs_f64(MAX_SECONDS)),
        ])
        .last()
        .inspect(|generation| {
            ascii_snake(SnakeAI::Graph(generation.value()));
            println!("{}", generation.metrics().dashboard());
        });
}

fn run_neural_net_engine() {
    let shapes = vec![(INPUT_SIZE, 16), (16, 16), (16, OUTPUT_SIZE)];
    let engine = GeneticEngine::builder()
        .codec(FloatCodec::tensor(shapes, -1.0..1.0).with_bounds(-5.0..5.0))
        .parallel()
        .alter(alters!(
            IntermediateCrossover::new(0.75, 0.1),
            ArithmeticMutator::new(0.03),
            GaussianMutator::new(0.03)
        ))
        .fitness_fn(|weights: Vec<Vec<Vec<f32>>>| snake_fitness(SnakeAI::NeuralNet(&weights)))
        .build();

    engine
        .iter()
        .logging()
        .limit(vec![
            Limit::Generation(MAX_GENERATIONS),
            Limit::Seconds(Duration::from_secs_f64(MAX_SECONDS)),
        ])
        .last()
        .inspect(|generation| {
            ascii_snake(SnakeAI::NeuralNet(generation.value()));
            println!("{}", generation.metrics().dashboard());
        });
}

fn snake_fitness<'a>(ai: SnakeAI<'a>) -> f32 {
    let mut total = 0.0;

    for _ in 0..NUM_GAMES {
        total += episode_return(&ai);
    }

    total / NUM_GAMES as f32
}

fn episode_return<'a>(ai: &SnakeAI<'a>) -> f32 {
    let mut game = SnakeGame::new(WIDTH, HEIGHT);

    let mut unique_cells = HashSet::new();
    unique_cells.insert(game.snake[0]);

    let mut recent_positions = vec![game.snake[0]];
    let mut last_direction = game.direction;

    let mut discounted_return = 0.0f32;
    let mut discount = 1.0f32;
    let mut last_score = 0i32;

    while !game.game_over {
        let action = ai.predict(&game.get_state());

        let (hx, hy) = game.snake[0];
        let old_dist = (game.food.0 - hx).abs() + (game.food.1 - hy).abs();

        let alive = game.step(action);

        let mut step_reward = 0.0f32;

        if !alive {
            if game.steps < MIN_SURVIVAL_STEPS && game.score == 0 {
                step_reward += EARLY_DEATH_PENALTY;
            }

            discounted_return += discount * step_reward;
            break;
        }

        let (nhx, nhy) = game.snake[0];
        let new_dist = (game.food.0 - nhx).abs() + (game.food.1 - nhy).abs();

        // Potential-based shaping
        step_reward += K_POTENTIAL * (old_dist as f32 - new_dist as f32);

        // Time penalty
        step_reward += STEP_PENALTY;

        // Jitter penalty
        if game.direction != last_direction {
            step_reward += JITTER_PENALTY;
        }

        last_direction = game.direction;

        // Exploration bonus
        if unique_cells.insert((nhx, nhy)) {
            step_reward += EXPLORATION_BONUS;
        }

        // Loop penalty
        if recent_positions.contains(&(nhx, nhy)) {
            step_reward += LOOP_PENALTY;
        }

        recent_positions.push((nhx, nhy));
        if recent_positions.len() > LOOP_WINDOW {
            recent_positions.remove(0);
        }

        // Eating bonus
        if game.score > last_score {
            step_reward += EAT_REWARD + 0.5 * game.snake.len() as f32;
            last_score = game.score;
        }

        discounted_return += discount * step_reward;
        discount *= GAMMA;
    }

    // Terminal score bonus
    discounted_return += (game.score.pow(2) as f32) * SCORE_TERMINAL;
    discounted_return.max(1.0)
}

fn draw_ascii_frame(game: &SnakeGame, step: usize) {
    let mut out = String::new();
    out.push_str("\x1B[2J\x1B[H"); // clear + home

    out.push_str(&format!(
        "ASCII SNAKE | score: {} | steps: {}\n",
        game.score, game.steps
    ));

    let height_with_border = game.height + 2;
    let width_with_border = game.width + 2;

    for y in 0..height_with_border {
        for x in 0..width_with_border {
            if x == 0 || x == width_with_border - 1 || y == 0 || y == height_with_border - 1 {
                out.push('#');
                continue;
            }

            let gx = x - 1;
            let gy = y - 1;

            if (gx, gy) == game.food {
                out.push('*');
                continue;
            }

            if game.snake[0] == (gx, gy) {
                out.push('@');
                continue;
            }

            if game.snake.iter().skip(1).any(|&p| p == (gx, gy)) {
                out.push('o');
                continue;
            }

            out.push(' ');
        }
        out.push('\n');
    }

    out.push_str(&format!("step: {}\n", step));

    print!("{out}");
    io::stdout().flush().unwrap();
}

fn ascii_snake<'a>(ai: SnakeAI<'a>) {
    let mut game = SnakeGame::new(WIDTH, HEIGHT);
    let max_steps = MAX_STEPS.min(10000);

    for step in 0..max_steps {
        if game.game_over {
            break;
        }

        let state = game.get_state();
        let action = ai.predict(&state);
        game.step(action);

        draw_ascii_frame(&game, step);
        thread::sleep(Duration::from_millis(35));
    }
}
