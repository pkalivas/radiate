use radiate::prelude::*;

const MIN_SCORE: f32 = 0.001;
const MAX_SECONDS: f64 = 15.0;
const MIN_TREE_DEPTH: usize = 3;

fn main() -> RadiateResult<()> {
    random_provider::seed(1000);

    let data = DataSet::pagie1().shuffle();
    let (_, in_dim, _) = data.shape();

    let set = node_set! {
        Root => Op::sum(),
        Vertex => [
            Op::abs(),
            Op::log(),
            Op::sigmoid(),
            Op::tanh(),
            Op::sin(),
            Op::mul(),
            Op::add(),
            Op::div(),
        ],
        Leaf => Op::vars(0..in_dim)
            .into_iter()
            .chain(std::iter::once(Op::erc()))
            .collect::<Vec<_>>()
    };

    let codec = TreeCodec::single(MIN_TREE_DEPTH, set).max_size(40);

    let engine = GeneticEngine::builder()
        .codec(codec)
        .fitness_fn(Regression::new(data.clone(), Loss::MSE))
        .minimizing()
        .diversity(TreeDistance::new())
        .target_species(5)
        .alter(alters!(
            TreeCrossover::new(0.5),
            OperationMutator::new(0.01, 0.08),
            SubtreeMutator::new(0.1),
        ))
        .build();

    let result = engine
        .iter()
        .logging()
        .until(|epoch| epoch.score().as_f32() < MIN_SCORE || epoch.seconds() > MAX_SECONDS)
        .last()?;

    display(&data, &result);

    Ok(())
}

fn display(train: &DataSet<f32>, result: &Generation<TreeChromosome<Op<f32>>, Tree<Op<f32>>>) {
    println!("{}", result.metrics().dashboard());
    let mut best = result.value().clone();

    let train_acc = Accuracy::default()
        .named("train")
        .on(train)
        .loss(Loss::MSE)
        .calc(&mut best);

    println!("{:?}", result);
    println!("{:?}", train_acc);

    let grid = DataSet::pagie1();
    plot(&grid, result.value());
}

fn plot(grid: &DataSet<f32>, tree: &Tree<Op<f32>>) {
    use plotly::common::{Line, Mode};
    use plotly::layout::{Axis, Layout};
    use plotly::{Plot, Scatter};

    let inputs = grid.features();
    let actual = grid.labels().iter().map(|row| row[0]).collect::<Vec<f32>>();
    let preds = inputs
        .iter()
        .map(|row| tree.eval(row))
        .collect::<Vec<f32>>();
    let index = (0..actual.len()).collect::<Vec<_>>();
    let labels = inputs
        .iter()
        .map(|row| format!("x0 = {:.1}, x1 = {:.1}", row[0], row[1]))
        .collect::<Vec<_>>();

    let line = |name: &str, y: Vec<f32>, color: &'static str| {
        Scatter::new(index.clone(), y)
            .name(name)
            .mode(Mode::Lines)
            .line(Line::new().color(color).width(2.0))
            .text_array(labels.clone())
            .hover_template(format!("{name} %{{y:.3}}<br>%{{text}}<extra></extra>"))
    };

    let mut plot = Plot::new();
    plot.add_trace(line("actual", actual, "#2a78d6"));
    plot.add_trace(line("predicted", preds, "#eb6834"));
    plot.set_layout(
        Layout::new()
            .title("Pagie-1: actual vs predicted")
            .x_axis(Axis::new().title("sample (x0-major grid order)"))
            .y_axis(Axis::new().title("y")),
    );
    plot.show();
}
