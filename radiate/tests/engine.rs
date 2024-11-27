#[cfg(test)]
mod engine_tests {
    use radiate::*;

    #[test]
    fn engine_can_minimize() {
        let codex = IntCodex::new(1, 5, 0, 100);

        let engine = GeneticEngine::from_codex(&codex)
            .minimizing()
            .fitness_fn(|genotype: Vec<Vec<i32>>| {
                Score::from_int(
                    genotype
                        .iter()
                        .fold(0, |acc, chromosome| acc + chromosome.iter().sum::<i32>()),
                )
            })
            .build();

        let result = engine.run(|output| output.score().as_int() == 0);

        let best = result.best.first().unwrap();
        assert_eq!(best.iter().sum::<i32>(), 0);
    }

    #[test]
    fn engine_can_maximize() {
        let codex = IntCodex::new(1, 5, 0, 101);

        let engine = GeneticEngine::from_codex(&codex)
            .fitness_fn(|genotype: Vec<Vec<i32>>| {
                Score::from_int(
                    genotype
                        .iter()
                        .fold(0, |acc, chromosome| acc + chromosome.iter().sum::<i32>()),
                )
            })
            .build();

        let result = engine.run(|output| output.score().as_int() == 500);

        let best = result.best.first().unwrap();
        assert_eq!(best.iter().sum::<i32>(), 500);
    }

    #[test]
    fn engine_evolves_towards_target() {
        let target = [1, 2, 3, 4, 5];
        let codex = IntCodex::new(1, target.len(), 0, 10);

        let engine = GeneticEngine::from_codex(&codex)
            .minimizing()
            .fitness_fn(move |genotype: Vec<Vec<i32>>| {
                let first = &genotype[0];
                let mut score = 0;
                for i in 0..first.len() {
                    score += (first[i] - target[i]).abs();
                }
                Score::from_int(score)
            })
            .build();

        let result = engine.run(|output| output.score().as_int() == 0);

        let best = result.best.first().unwrap();
        assert_eq!(best, &vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn population_initialization() {
        // let codex = IntCodex::new(1, 5, 0, 100);
        // let engine = GeneticEngine::from_codex(&codex)
        //     .population_size(150)
        //     .fitness_fn(|_| Score::from_int(1))
        //     .build();

        // let result = engine.run(|_| true);
        // assert_eq!(result.population.len(), 150);
        // for individual in result.population {
        //     assert!(individual.is_valid());
        // }

        let codex = FloatCodex::new(1, 5, 0.0, 100.0);
 // This codex will encode Genotype instances with 1 Chromosome and 5 FloatGenes,
 // with random allels between 0.0 and 100.0. It will decode into a Vec<Vec<f32>>.
 // eg: [[1.0, 2.0, 3.0, 4.0, 5.0]]

 // Create a new instance of the genetic engine with the given codex.
    let engine = GeneticEngine::from_codex(&codex)
        .minimizing()  // Minimize the fitness function.
        .population_size(150) // Set the population size to 150 individuals.
        .max_age(15) // Set the maximum age of an individual to 15 generations before it is replaced with a new individual.
        .offspring_fraction(0.5) // Set the fraction of the population that will be replaced by offspring each generation.
        .num_threads(4) // Set the number of threads to use in the thread pool for parallel fitness evaluation.
        .offspring_selector(BoltzmannSelector::new(4_f32)) // Use boltzmann selection to select offspring.
        .survivor_selector(TournamentSelector::new(3)) // Use tournament selection to select survivors.
        .alterer(alters![
            NumericMutator::new(0.01), // Specific mutator for numeric values.
            MeanCrossover::new(0.5) // Specific crossover operation for numeric values.
        ])
        .fitness_fn(|genotype: Vec<Vec<f32>>| { // Define the fitness function to be minimized.
            // Calculate the fitness score of the individual based on the decoded genotype.
            let score = genotype.iter().fold(0.0, |acc, chromosome| {
                acc + chromosome.iter().sum::<f32>()
            });
            Score::from_f32(score)
        })
    .build(); 

 // Run the genetic algorithm until the score of the best individual is 0, then return the result.
 let result = engine.run(|output| output.score().as_int() == 0);
    }
}
