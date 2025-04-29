use plotly::{Plot, Scatter3D};
use radiate::*;

const VARIABLES: usize = 4;
const OBJECTIVES: usize = 3;
const K: usize = VARIABLES - OBJECTIVES + 1;

fn main() {
    random_provider::set_seed(501);

    let codex = FloatCodex::vector(VARIABLES, 0_f32..1_f32).with_bounds(-100.0..100.0);

    let engine = GeneticEngine::from_codex(codex)
        .num_threads(10)
        .multi_objective(vec![Optimize::Minimize; OBJECTIVES])
        // .front_size(1100..1300)
        .offspring_selector(TournamentSelector::new(5))
        .survivor_selector(NSGA2Selector::new())
        .alter(alters!(
            SimulatedBinaryCrossover::new(1_f32, 1.0),
            UniformMutator::new(0.1_f32),
        ))
        .fitness_fn(|geno: Vec<f32>| dtlz_1(&geno))
        .build();

    let result = engine
        .iter()
        .take(1000)
        .inspect(|ctx| {
            println!("[ {:?} ]", ctx.index);
        })
        .last()
        .unwrap();

    println!("{:?}", result.seconds());
    println!("{:?}", result.metrics);
    plot_front(&result.front.read().unwrap());
}

fn plot_front(front: &Front<Phenotype<FloatChromosome>>) {
    let mut x = vec![];
    let mut y = vec![];
    let mut z = vec![];
    let mut color = vec![];

    println!("Front size: {:?}", front.values().len());

    for (i, pheno) in front.values().iter().enumerate() {
        let score = pheno.score().unwrap();
        x.push(score.values[0]);
        y.push(score.values[1]);
        z.push(score.values[2]);
        color.push(i as f32);
    }

    let mut plot = Plot::new();
    let trace = Scatter3D::new(x, y, z)
        .name("Pareto Front")
        .mode(plotly::common::Mode::Markers)
        .marker(plotly::common::Marker::new().color_array(color).size(4));

    plot.add_trace(trace);
    plot.show();
}

pub fn dtlz_1(values: &[f32]) -> Vec<f32> {
    let g = values[K..]
        .iter()
        .map(|&xi| (xi - 0.5).powi(2))
        .sum::<f32>();

    let f1 = (1.0 + g) * (values[0] * values[1]);
    let f2 = (1.0 + g) * (values[0] * (1.0 - values[1]));
    let f3 = (1.0 + g) * (1.0 - values[0]);

    vec![f1, f2, f3]
}

pub fn dtlz_2(values: &[f32]) -> Vec<f32> {
    let g = values[K..]
        .iter()
        .map(|&xi| (xi - 0.5).powi(2))
        .sum::<f32>();

    let mut f = vec![1.0 + g; OBJECTIVES];
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
    let k = VARIABLES - OBJECTIVES + 1;
    let g: f32 = values[VARIABLES - k..].iter().map(|&xi| xi.powf(0.1)).sum();

    let mut f = vec![1.0 + g; OBJECTIVES];
    let theta = std::f32::consts::PI / (4.0 * (1.0 + g));

    for i in 0..OBJECTIVES {
        for j in 0..OBJECTIVES - (i + 1) {
            f[i] *= (values[j] * theta).cos();
        }
        if i != 0 {
            let aux = OBJECTIVES - (i + 1);
            f[i] *= (values[aux] * theta).sin();
        }
    }
    f
}

pub fn dtlz_7(values: &[f32]) -> Vec<f32> {
    let k = VARIABLES - OBJECTIVES + 1;
    let g: f32 = 1.0 + (9.0 / k as f32) * values[OBJECTIVES - 1..].iter().sum::<f32>();

    let mut f = vec![0.0; OBJECTIVES];

    for i in 0..OBJECTIVES - 1 {
        f[i] = values[i];
    }

    let h: f32 = OBJECTIVES as f32
        - values[..OBJECTIVES - 1]
            .iter()
            .map(|&x| x / (1.0 + g) * (1.0 + (3.0 * std::f32::consts::PI * x).sin()))
            .sum::<f32>();

    f[OBJECTIVES - 1] = (1.0 + g) * h;

    f
}
