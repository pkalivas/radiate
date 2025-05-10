use radiate::{Chromosome, Generation, GeneticEngineBuilder};

pub(crate) fn set_single_objective<C, T>(
    builder: GeneticEngineBuilder<C, T>,
    objectives: &[String],
) -> GeneticEngineBuilder<C, T, Generation<C, T>>
where
    C: Chromosome,
    T: Clone + Send + Sync,
{
    if objectives.len() != 1 {
        panic!("Single objective optimization requires exactly one objective");
    }

    let obj = objectives[0].to_lowercase();

    match obj.as_str() {
        "min" => builder.minimizing(),
        "max" => builder.maximizing(),
        _ => panic!("Invalid objective: {}", obj),
    }
}
