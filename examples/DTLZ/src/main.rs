use plotly::{Plot, Scatter3D};
use radiate::objectives::{Front, Optimize};
use radiate::*;

const VARIABLES: usize = 4;
const OBJECTIVES: usize = 3;
const K: usize = VARIABLES - OBJECTIVES + 1;

fn main() {
    let codex = FloatCodex::new(1, VARIABLES, 0_f32, 1_f32).with_bounds(0.0, 1.0);

    let engine = GeneticEngine::from_codex(&codex)
        .num_threads(4)
        .multi_objective(vec![Optimize::Minimize; OBJECTIVES])
        .front_size(1000, 1100)
        .offspring_selector(TournamentSelector::new(5))
        .survivor_selector(NSGA2Selector::new())
        .alter(alters!(
            SimulatedBinaryCrossover::new(1_f32, 1.0),
            UniformMutator::new(0.1_f32),
        ))
        .fitness_fn(|geno: Vec<Vec<f32>>| dtlz_7(geno.first().unwrap()))
        .build();

    let result = engine.run(|output| {
        println!("[ {:?} ]", output.index);
        output.index > 1000
    });

    let front = result.front.lock().unwrap();
    println!("{:?}", result.metrics);
    plot_front(&front);
}

fn plot_front(front: &Front) {
    let mut x = vec![];
    let mut y = vec![];
    let mut z = vec![];

    for score in front.scores().iter() {
        x.push(score.values[0]);
        y.push(score.values[1]);
        z.push(score.values[2]);
    }

    let mut plot = Plot::new();
    let trace = Scatter3D::new(x, y, z)
        .name("Front")
        .mode(plotly::common::Mode::Markers);

    plot.add_trace(trace);
    plot.show();
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
