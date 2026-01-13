use radiate::*;

fn main() {
    random_provider::set_seed(40);

    let data = PgmDataSet::new(vec![
        vec![Some(0), Some(1), Some(0)],
        vec![Some(1), Some(1), Some(0)],
        vec![Some(0), Some(0), Some(1)],
        vec![Some(1), Some(0), Some(1)],
        vec![Some(0), Some(1), Some(1)],
        vec![Some(1), Some(1), Some(1)],
    ]);

    // 1) Define the PGM structure
    //   - 3 variables, each with 2 states - binary variables
    let cards = [2, 2, 2];

    // 2) Build the codec for PGMs with 6 factors and max scope size 3
    let codec = PgmCodec::new(&cards, 6, 3);

    // 3) Define the fitness function - log-likelihood on the data
    let fitness = PgmLogLik::new(data);

    let engine = GeneticEngine::builder()
        .codec(codec)
        .fitness_fn(fitness)
        .minimizing()
        .alter(alters!(
            PgmScopeMutator::new(0.05, 3),
            PgmParamMutator::new(
                0.50, // half the factors get touched per individual
                0.10, // mutate ~10% of table entries
                0.25, // jitter magnitude
            )
        ))
        .build();

    engine
        .iter()
        .logging()
        .take(300)
        .last()
        .inspect(|generation| {
            println!("{}", generation.metrics().dashboard());
            for i in generation.value().iter() {
                println!("Factor  {:?}", i);
                println!("");
            }
        });
}
