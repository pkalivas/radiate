use radiate::{Chromosome, Generation, GeneticEngineBuilder, MultiObjectiveGeneration, Optimize};

pub(crate) fn set_single_objective<C, T>(
    builder: GeneticEngineBuilder<C, T, Generation<C, T>>,
    objectives: &[String],
) -> GeneticEngineBuilder<C, T, Generation<C, T>>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send + Sync,
{
    let obj = objectives[0].to_lowercase();

    match obj.as_str() {
        "min" => builder.minimizing(),
        "max" => builder.maximizing(),
        _ => panic!("Invalid objective: {}", obj),
    }
}

pub(crate) fn set_multi_objective<C, T>(
    builder: GeneticEngineBuilder<C, T, Generation<C, T>>,
    objectives: &[String],
) -> GeneticEngineBuilder<C, T, MultiObjectiveGeneration<C>>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send + Sync,
{
    builder.multi_objective(
        objectives
            .iter()
            .map(|ob| match ob.to_lowercase().trim() {
                "min" => Optimize::Minimize,
                "max" => Optimize::Maximize,
                _ => panic!("Invalid objective {}", ob),
            })
            .collect::<Vec<Optimize>>(),
    )
}
