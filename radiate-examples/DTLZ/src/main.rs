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
        .alterer(alters!(
            SimulatedBinaryCrossover::new(1_f32, 1.0),
            UniformMutator::new(1.0 / VARIABLES as f32)
        ))
        .fitness_fn(move |genotype: Vec<Vec<f32>>| {
            let f = DTLZ_7(genotype.first().unwrap().clone());
            Score::from_vec(f)
        })
        .build();

    let result = engine.run(move |output| {
        println!("[ {:?} ]: {:?}", output.index, output.score());

        output.index > 2500
    });

    let front = result.front.lock().unwrap();
    println!("{:?}", result.metrics);
    write_front(&front);
}

fn write_front(front: &Front) {
    let current_dir = std::env::current_dir().unwrap();
    let full_path = current_dir.join("radiate-examples/DTLZ/front.csv");
    let mut file = std::fs::File::create(full_path).unwrap();
    write!(file, "f1,f2,f3,").unwrap();
    writeln!(file).unwrap();
    for score in front.scores().iter() {
        for value in score.values.iter() {
            write!(file, "{},", value).unwrap();
        }
        writeln!(file).unwrap();
    }
}

pub fn DTLZ_1(values: Vec<f32>) -> Vec<f32> {
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

pub fn DTLZ_2(values: Vec<f32>) -> Vec<f32> {
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

pub fn DTLZ_6(values: Vec<f32>) -> Vec<f32> {
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

pub fn DTLZ_7(values: Vec<f32>) -> Vec<f32> {
    let mut g = vec![0.0; OBJECTIVES];
    let mut x = vec![0.0; VARIABLES - K + 1];

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
    for i in 0..OBJECTIVES - 1 {
        f[i] = values[i];
    }

    f[OBJECTIVES - 1] = h * (1.0 + g[OBJECTIVES - 1]);

    f
}

// public static float[] FitnessDTLZ1(float[] values)
// {
// var g = 0f;
// for (var i = Variables - K; i < Variables; i++)
// {
// g += (float) Math.Pow(values[i] - 0.5f, 2f) - (float) Math.Cos(20f * Math.PI * (values[i] - 0.5f));
// }
//
// g = 100f * (K + g);
//
// var f = new float[Objectives];
// for (var i = 0; i < Objectives; i++)
// {
// f[i] = 0.5f * (1f + g);
// for (var j = 0; j < Objectives - 1 - i; j++)
// {
// f[i] *= values[j];
// }
//
// if (i != 0)
// {
// f[i] *= 1f - values[Objectives - 1 - i];
// }
// }
//
// return f;
// }
//
// public static float[] FitnessDTLZ2(float[] values)
// {
// var g = 0.0f;
// for (var i = Variables - K; i < Variables; i++)
// {
// g += (float) Math.Pow(values[i] - 0.5f, 2f) - (float) Math.Cos(20f * Math.PI * (values[i] - 0.5f));
// }
// g = 100.0f * (K + g);
//
// var f = new float[Objectives];
//
// for (var i = 0; i < Objectives; i++)
// {
// f[i] = 1.0f + g;
// }
//
// for (var i = 0; i < Objectives; i++)
// {
// for (var j = 0; j < Objectives - (i + 1); j++)
// {
// f[i] *= (float) Math.Cos(values[j] * 0.5 * Math.PI);
// }
// if (i != 0)
// {
// var aux = Objectives - (i + 1);
// f[i] *= (float)Math.Sin(values[aux] * 0.5 * Math.PI);
// }
// }
//
// return f;
// }
//
// public float[] FitnessDTZL6(float[] decisionVariables)
// {
// var f = new float[Objectives];
// var k = Variables - Objectives + 1;
//
// var g = 0.0f;
// for (var i = Variables - k; i < Variables; i++)
// {
// g += (float) Math.Pow(decisionVariables[i], 0.1);
// }
//
// var theta = (float) Math.PI / (4.0 * (1.0 + g));
//
// for (var i = 0; i < Objectives; i++)
// {
// f[i] = (1.0f + g) * (float) Math.Cos((1.0 + 2.0 * g) * Math.PI * 0.5);
// }
//
// for (var i = 0; i < Objectives; i++)
// {
// for (var j = 0; j < Objectives - (i + 1); j++)
// {
// f[i] *= (float) Math.Cos(theta * decisionVariables[j] * Math.PI * 0.5);
// }
// if (i != 0)
// {
// var aux = Objectives - (i + 1);
// f[i] *= (float) Math.Sin(theta * decisionVariables[aux] * Math.PI * 0.5);
// }
// }
//
// return f;
// }
//
// public float[] FitnessDTZL7(float[] decisionVariables)
// {
// var g = new float[Objectives];
// var x = new float[Variables - K + 1];
//
// for (var i = Variables - K; i < Variables; i++)
// {
// x[i - (Variables - K)] = decisionVariables[i];
// }
//
// var sum = 0f;
// for (var i = 0; i < x.Length; i++)
// {
// sum += x[i];
// }
//
// var h = 0f;
// for (var i = 0; i < Objectives; i++)
// {
// g[i] = 1f + 9.0f / K * sum;
// }
//
// for (var i = 0; i < Objectives - 1; i++)
// {
// h += decisionVariables[i] / (1f + g[i]) * (float) (1f + Math.Sin(3f * Math.PI * decisionVariables[i]));
// }
//
// h = Objectives - h;
//
// var f = new float[Objectives];
// for (var i = 0; i < Objectives - 1; i++)
// {
// f[i] = decisionVariables[i];
// }
//
// f[Objectives - 1] = h * (1 + g[Objectives - 1]);
//
// return f;
// }
