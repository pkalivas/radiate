//! Koza's 6-multiplexer: evolve a boolean program that uses two address bits to pick one of
//! four data bits.
//!
//! ```text
//!   inputs:  a0 a1 d0 d1 d2 d3        output = d[a0 + 2·a1]
//! ```
//!
//! There are 2⁶ = 64 input combinations, and a tree's score is how many it gets wrong, so 0
//! means it computes the multiplexer exactly. A textbook solution is nested if/else:
//!
//! ```text
//!   if a1 (if a0 d3 d2) (if a0 d1 d0)
//! ```
use radiate::prelude::*;

const ADDRESS_BITS: usize = 2;
const DATA_BITS: usize = 1 << ADDRESS_BITS;
const INPUTS: usize = ADDRESS_BITS + DATA_BITS;
const MAX_SECONDS: f64 = 10.0;

fn main() -> RadiateResult<()> {
    random_provider::seed(42);

    let cases = truth_table();
    let table = cases.clone();

    // Koza's original function set: and, or, not, if/else.
    let set = node_set! {
        Vertex => [Op::and(), Op::or(), Op::not(), Op::if_else()],
        Leaf => Op::vars(0..INPUTS)
    };

    let codec = TreeCodec::single(2, set).constraint(|node| node.size() < 50);

    let engine = GeneticEngine::builder()
        .codec(codec)
        .fitness_fn(move |tree: Tree<Op<bool>>| errors(&tree, &table))
        .minimizing()
        .diversity(TreeDistance::new())
        .target_species(5)
        .alter(alters!(
            TreeCrossover::new(0.5),
            HoistMutator::new(0.1),
            OperationMutator::new(0.03, 0.02),
        ))
        .build();

    let result = engine
        .iter()
        .logging()
        .until(|epoch| epoch.score().as_f32() == 0.0 || epoch.seconds() > MAX_SECONDS)
        .last()?;

    display(&result, &cases);

    Ok(())
}

fn truth_table() -> Vec<(Vec<bool>, bool)> {
    (0..1usize << INPUTS)
        .map(|bits| {
            let inputs = (0..INPUTS).map(|i| bits >> i & 1 == 1).collect::<Vec<_>>();
            let address = (0..ADDRESS_BITS).fold(0, |acc, i| acc | (inputs[i] as usize) << i);
            let output = inputs[ADDRESS_BITS + address];
            (inputs, output)
        })
        .collect()
}

/// How many input combinations the tree gets wrong.
fn errors(tree: &Tree<Op<bool>>, cases: &[(Vec<bool>, bool)]) -> f32 {
    cases
        .iter()
        .filter(|(inputs, output)| tree.eval(inputs) != *output)
        .count() as f32
}

fn display(
    result: &Generation<TreeChromosome<Op<bool>>, Tree<Op<bool>>>,
    cases: &[(Vec<bool>, bool)],
) {
    let tree = result.value();
    let wrong = errors(tree, cases) as usize;

    println!("{}", result.metrics().dashboard());
    println!("Inputs: X0 = a0, X1 = a1, X2..X5 = d0..d3\n");
    println!("Best tree ({} nodes):\n{}", tree.size(), tree.format());
    println!(
        "Correct on {} of {} cases after {} generations ({:.2}s)",
        cases.len() - wrong,
        cases.len(),
        result.index(),
        result.seconds()
    );
}
