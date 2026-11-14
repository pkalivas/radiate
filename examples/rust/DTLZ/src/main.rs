use plotly::{Layout, Plot, Scatter3D, layout::Margin};
use radiate::prelude::*;

const VARIABLES: usize = 4;
const OBJECTIVES: usize = 3;
const K: usize = VARIABLES - OBJECTIVES + 1;

fn main() {
    random_provider::seed(500);

    let codec = FloatCodec::vector(VARIABLES, 0_f32..1_f32);

    let engine = GeneticEngine::builder()
        .codec(codec)
        .fitness_fn(|geno: Vec<f32>| dtlz_1(&geno))
        .multi_objective(vec![Optimize::Minimize; OBJECTIVES])
        // NSGA-III for 3+ objectives: crowded-comparison tournament for parents,
        // reference-point niching for survivors. For 2 objectives use `NSGA2Selector::new()`.
        .offspring_selector(TournamentNSGA2Selector::new())
        .survivor_selector(NSGA3Selector::new(12))
        // A lower offspring fraction keeps more of the already-evaluated front each generation.
        .offspring_fraction(0.5)
        .front_size(200..250)
        .alter(alters!(
            SimulatedBinaryCrossover::new(0.8_f32, 20_f32),
            PolynomialMutator::new(0.1, 20.0),
        ))
        .build();

    let result = radiate::ui(engine)
        .iter()
        .logging()
        .limit(1000)
        .last()
        .unwrap();

    println!("{:?}", result);
    println!("{}", result.metrics().dashboard());
    let front = result.front().unwrap();

    // DTLZ1's Pareto front is the plane f1 + f2 + f3 = 0.5, so with a reference point of
    // 1.0 in every objective the best achievable hypervolume is 1 - 0.5³/6 ≈ 0.9792.
    let reference = [1.0; OBJECTIVES];
    if let Some(hypervolume) = front.hypervolume(&reference) {
        println!("Hypervolume: {hypervolume:.4}");
    }

    plot_front(front);
}

fn plot_front(front: &Front<Phenotype<FloatChromosome<f32>>>) {
    let mut front = front.clone();
    front.remove_outliers(0.05);
    let mut x = vec![];
    let mut y = vec![];
    let mut z = vec![];
    let mut color = vec![];

    for (i, pheno) in front.values().iter().enumerate() {
        let score = pheno.score().unwrap();
        x.push(score[0]);
        y.push(score[1]);
        z.push(score[2]);
        color.push(i as f32);
    }

    let mut plot = Plot::new();
    plot.set_layout(
        Layout::new()
            .title("DTLZ Pareto Front")
            .margin(Margin::new().left(0).right(0).top(0).bottom(0))
            .scene(plotly::layout::LayoutScene::new()),
    );
    let trace = Scatter3D::new(x, y, z)
        .name("Pareto Front")
        .mode(plotly::common::Mode::Markers)
        .marker(plotly::common::Marker::new().color_array(color).size(4));

    plot.add_trace(trace);
    plot.show();
}

pub fn dtlz_1(values: &[f32]) -> Vec<f32> {
    let mut g = 0.0;
    for x in &values[VARIABLES - K..VARIABLES] {
        g += (x - 0.5).powi(2) - (20.0 * std::f32::consts::PI * (x - 0.5)).cos();
    }

    g = 100.0 * (K as f32 + g);

    let mut f = vec![0.0; OBJECTIVES];
    for (i, fi) in f.iter_mut().enumerate() {
        *fi = 0.5 * (1.0 + g);
        for x in &values[..OBJECTIVES - 1 - i] {
            *fi *= x;
        }

        if i != 0 {
            *fi *= 1.0 - values[OBJECTIVES - 1 - i];
        }
    }

    f
}

pub fn dtlz_2(values: &[f32]) -> Vec<f32> {
    let g = values[K..]
        .iter()
        .map(|&xi| (xi - 0.5).powi(2))
        .sum::<f32>();

    let mut f = vec![1.0 + g; OBJECTIVES];
    for (i, fi) in f.iter_mut().enumerate() {
        for x in &values[..OBJECTIVES - (i + 1)] {
            *fi *= (x * 0.5 * std::f32::consts::PI).cos();
        }
        if i != 0 {
            let aux = OBJECTIVES - (i + 1);
            *fi *= (values[aux] * 0.5 * std::f32::consts::PI).sin();
        }
    }

    f
}

pub fn dtlz_6(values: &[f32]) -> Vec<f32> {
    let k = VARIABLES - OBJECTIVES + 1;
    let g: f32 = values[VARIABLES - k..].iter().map(|&xi| xi.powf(0.1)).sum();

    let mut f = vec![1.0 + g; OBJECTIVES];
    let theta = std::f32::consts::PI / (4.0 * (1.0 + g));

    for (i, fi) in f.iter_mut().enumerate() {
        for x in &values[..OBJECTIVES - (i + 1)] {
            *fi *= (x * theta).cos();
        }
        if i != 0 {
            let aux = OBJECTIVES - (i + 1);
            *fi *= (values[aux] * theta).sin();
        }
    }

    f
}

pub fn dtlz_7(values: &[f32]) -> Vec<f32> {
    let k = VARIABLES - OBJECTIVES + 1;
    let g: f32 = 1.0 + (9.0 / k as f32) * values[OBJECTIVES - 1..].iter().sum::<f32>();

    let mut f = vec![0.0; OBJECTIVES];

    f[..OBJECTIVES - 1].copy_from_slice(&values[..OBJECTIVES - 1]);

    let h: f32 = OBJECTIVES as f32
        - values[..OBJECTIVES - 1]
            .iter()
            .map(|&x| x / (1.0 + g) * (1.0 + (3.0 * std::f32::consts::PI * x).sin()))
            .sum::<f32>();

    f[OBJECTIVES - 1] = (1.0 + g) * h;

    f
}
