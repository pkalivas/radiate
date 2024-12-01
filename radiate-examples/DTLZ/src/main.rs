use radiate::objectives::{Front, Optimize};
use radiate::*;
use std::io::Write;

const VARIABLES: usize = 4;
const OBJECTIVES: usize = 3;
const K: usize = VARIABLES - OBJECTIVES + 1;

fn main() {
    let codex = FloatCodex::new(1, VARIABLES, 0_f32, 1_f32).with_bounds(0.0, 1.0);

    let engine = GeneticEngine::from_codex(&codex)
        .population_size(100)
        .num_threads(10)
        .multi_objective(vec![Optimize::Minimize; OBJECTIVES])
        .offspring_selector(TournamentSelector::new(5))
        .survivor_selector(NSGA2Selector::new())
        .alter(alters!(
            SimulatedBinaryCrossover::new(1_f32, 1.0),
            UniformMutator::new(0.1_f32),
        ))
        .fitness_fn(move |genotype: Vec<Vec<f32>>| {
            let f = dtlz_6(genotype.first().unwrap());
            Score::from_vec(f)
        })
        .build();

    let result = engine.run(move |output| {
        println!("[ {:?} ]: {:?}", output.index, output.score());

        output.index > 1000
    });

    let front = result.front.lock().unwrap();
    println!("{:?}", result.metrics);
    write_front(&front);
}

fn write_front(front: &Front) {
    let current_dir = std::env::current_dir().unwrap();
    let full_path = current_dir.join("radiate-examples/DTLZ/front.csv");
    let mut file = std::fs::File::create(full_path).unwrap();
    write!(file, "x,y,z,").unwrap();
    writeln!(file).unwrap();
    for score in front.scores().iter() {
        for value in score.values.iter() {
            write!(file, "{},", value).unwrap();
        }
        writeln!(file).unwrap();
    }
}

pub fn dtlz_1(values: &[f32]) -> Vec<f32> {
    let mut g = 0.0;
    for i in VARIABLES - K..VARIABLES {
        g += (values[i] - 0.5).powi(2) - (20.0 * std::f32::consts::PI * (values[i] - 0.5)).cos();
    }

    g = 100.0 * (K as f32 + g);

    let mut f = vec![0.0; OBJECTIVES];
    for i in 0..OBJECTIVES {
        f[i] = 0.5 * (1.0 + g);
        for j in 0..OBJECTIVES - 1 - i {
            f[i] *= values[j];
        }

        if i != 0 {
            f[i] *= 1.0 - values[OBJECTIVES - 1 - i];
        }
    }

    f
}

pub fn dtlz_2(values: &[f32]) -> Vec<f32> {
    let mut g = 0.0;
    for i in VARIABLES - K..VARIABLES {
        g += (values[i] - 0.5).powi(2) - (20.0 * std::f32::consts::PI * (values[i] - 0.5)).cos();
    }
    g = 100.0 * (K as f32 + g);

    let mut f = vec![0.0; OBJECTIVES];

    for i in 0..OBJECTIVES {
        f[i] = 1.0 + g;
    }

    for i in 0..OBJECTIVES {
        for j in 0..OBJECTIVES - (i + 1) {
            f[i] *= (values[j] * 0.5 * std::f32::consts::PI).cos();
        }
        if i != 0 {
            let aux = OBJECTIVES - (i + 1);
            f[i] *= (values[aux] * 0.5 * std::f32::consts::PI).sin();
        }
    }

    f
}

pub fn dtlz_6(values: &[f32]) -> Vec<f32> {
    let mut f = vec![0.0; OBJECTIVES];
    let k = VARIABLES - OBJECTIVES + 1;

    let mut g = 0.0;
    for i in VARIABLES - k..VARIABLES {
        g += values[i].powf(0.1);
    }

    let theta = std::f32::consts::PI / (4.0 * (1.0 + g));

    for i in 0..OBJECTIVES {
        f[i] = (1.0 + g) * (0.5 * (1.0 + 2.0 * g) * std::f32::consts::PI).cos();
    }

    for i in 0..OBJECTIVES {
        for j in 0..OBJECTIVES - (i + 1) {
            f[i] *= (theta * values[j] * std::f32::consts::PI * 0.5).cos();
        }
        if i != 0 {
            let aux = OBJECTIVES - (i + 1);
            f[i] *= (theta * values[aux] * std::f32::consts::PI * 0.5).sin();
        }
    }

    f
}

pub fn dtlz_7(values: &[f32]) -> Vec<f32> {
    let mut g = [0.0; OBJECTIVES];
    let mut x = [0.0; VARIABLES - K + 1];

    for i in VARIABLES - K..VARIABLES {
        x[i - (VARIABLES - K)] = values[i];
    }

    let mut sum = 0.0;
    for i in 0..x.len() {
        sum += x[i];
    }

    let mut h = 0.0;
    for i in 0..OBJECTIVES {
        g[i] = 1.0 + 9.0 / K as f32 * sum;
    }

    for i in 0..OBJECTIVES - 1 {
        h += values[i] / (1.0 + g[i]) * (1.0 + (3.0 * std::f32::consts::PI * values[i]).sin());
    }

    h = OBJECTIVES as f32 - h;

    let mut f = vec![0.0; OBJECTIVES];
    f[..(OBJECTIVES - 1)].copy_from_slice(&values[..(OBJECTIVES - 1)]);

    f[OBJECTIVES - 1] = h * (1.0 + g[OBJECTIVES - 1]);

    f
}
