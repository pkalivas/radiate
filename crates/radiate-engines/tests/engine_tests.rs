#[cfg(test)]
mod engine_tests {
    use radiate_core::*;
    use radiate_engines::*;

    /// Test problem: Evolve functions that produce diverse output patterns
    /// We want to find functions that behave differently, not necessarily optimally
    #[derive(Clone)]
    pub struct FunctionDiversityProblem {
        pub codec: FloatCodec<Vec<f32>>,
        pub test_inputs: Vec<f32>,
    }

    impl FunctionDiversityProblem {
        pub fn new(codec: FloatCodec<Vec<f32>>, test_inputs: Vec<f32>) -> Self {
            Self { codec, test_inputs }
        }
    }

    impl Problem<FloatChromosome, Vec<f32>> for FunctionDiversityProblem {
        fn encode(&self) -> Genotype<FloatChromosome> {
            self.codec.encode()
        }

        fn decode(&self, genotype: &Genotype<FloatChromosome>) -> Vec<f32> {
            self.codec.decode(genotype)
        }

        fn eval(&self, individual: &Genotype<FloatChromosome>) -> Score {
            let weights = self.decode(individual);

            // Create a simple function: f(x) = sum(weights[i] * x^i)
            let outputs: Vec<f32> = self
                .test_inputs
                .iter()
                .map(|&x| {
                    weights
                        .iter()
                        .enumerate()
                        .map(|(i, &w)| w * x.powi(i as i32))
                        .sum()
                })
                .collect();

            let target = vec![1.0, 2.0, 3.0, 4.0, 5.0];
            let fitness = 1.0
                / (1.0
                    + outputs
                        .iter()
                        .zip(target.iter())
                        .map(|(a, b)| (a - b).powi(2))
                        .sum::<f32>());

            Score::from(fitness)
        }
    }

    #[test]
    fn engine_can_minimize() {
        let engine = GeneticEngine::builder()
            .minimizing()
            .codec(IntCodec::vector(5, 0..100))
            .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
            .build();

        let result = engine.iter().until_score(0).take(1).last().unwrap();

        let best = result.value();
        assert_eq!(best.iter().sum::<i32>(), 0);
    }

    #[test]
    fn engine_can_maximize() {
        let mut engine = GeneticEngine::builder()
            .codec(IntCodec::vector(5, 0..101))
            .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
            .build();

        let result = engine.run(|ctx| ctx.score().as_i32() == 500);

        assert_eq!(result.value().iter().sum::<i32>(), 500);
    }

    #[test]
    fn engine_evolves_towards_target() {
        let target = [1, 2, 3, 4, 5];

        let mut engine = GeneticEngine::builder()
            .minimizing()
            .codec(IntCodec::vector(target.len(), 0..10))
            .fitness_fn(move |geno: Vec<i32>| {
                let mut score = 0;
                for i in 0..geno.len() {
                    score += (geno[i] - target[i]).abs();
                }
                score
            })
            .build();

        let result = engine.run(|ctx| ctx.score().as_i32() == 0);

        assert_eq!(result.value(), &vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_novelty_search() {
        use radiate_core::*;

        random_provider::set_seed(42);

        let test_inputs = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let codec = FloatCodec::vector(5, -100.0..100.0);

        let base_problem = FunctionDiversityProblem::new(codec.clone(), test_inputs);

        let base_population = (0..100)
            .map(|_| Phenotype::from(base_problem.encode()))
            .collect::<Population<FloatChromosome>>();

        let regular_engine = GeneticEngine::builder()
            .problem(base_problem.clone())
            .population(&base_population)
            .survivor_selector(TournamentSelector::new(3))
            .offspring_selector(RouletteSelector::new())
            .alter(alters![
                UniformCrossover::new(0.7),
                GaussianMutator::new(0.2),
            ])
            .minimizing()
            .fitness_fn(|geno: Vec<f32>| {
                let target = vec![1.0, 2.0, 3.0, 4.0, 5.0];
                1.0 / (1.0
                    + geno
                        .iter()
                        .zip(target.iter())
                        .map(|(a, b)| (a - b).powi(2))
                        .sum::<f32>())
            })
            .build();

        let novelty_engine = GeneticEngine::builder()
            .codec(codec.clone())
            .population(&base_population)
            .survivor_selector(TournamentSelector::new(3))
            .offspring_selector(RouletteSelector::new())
            .alter(alters![
                UniformCrossover::new(0.7),
                GaussianMutator::new(0.2),
            ])
            .minimizing()
            .fitness_fn(NoveltySearch::new(CosineDistance, 10, 0.03))
            .build();

        let combined_engine = GeneticEngine::builder()
            .codec(codec)
            .population(&base_population)
            .survivor_selector(TournamentSelector::new(3))
            .offspring_selector(RouletteSelector::new())
            .alter(alters![
                UniformCrossover::new(0.7),
                GaussianMutator::new(0.2),
            ])
            .minimizing()
            .fitness_fn(
                CompositeFitnessFn::new()
                    .add_weighted_fn(
                        |geno: &Vec<f32>| {
                            let target = vec![1.0, 2.0, 3.0, 4.0, 5.0];
                            1.0 / (1.0
                                + geno
                                    .iter()
                                    .zip(target.iter())
                                    .map(|(a, b)| (a - b).powi(2))
                                    .sum::<f32>())
                        },
                        0.5,
                    )
                    .add_weighted_fn(NoveltySearch::new(CosineDistance, 10, 0.03), 0.5),
            )
            .build();

        let regular_generation = regular_engine.iter().take(20).last().unwrap();
        let novelty_generation = novelty_engine.iter().take(20).last().unwrap();
        let combined_generation = combined_engine.iter().take(20).last().unwrap();

        let regular_diversity = calculate_diversity(regular_generation.population());
        let novelty_diversity = calculate_diversity(novelty_generation.population());
        let combined_diversity = calculate_diversity(combined_generation.population());

        assert!(novelty_diversity > regular_diversity);
        assert!(combined_diversity > regular_diversity);
        assert!(novelty_diversity > combined_diversity);
    }

    fn calculate_diversity(population: &Population<FloatChromosome>) -> f32 {
        let descriptors: Vec<Vec<f32>> = population
            .iter()
            .map(|individual| {
                let genotype = individual.genotype();
                genotype
                    .iter()
                    .flat_map(|chromosome| chromosome.iter().map(|g| *g.allele()))
                    .collect()
            })
            .collect();

        if descriptors.is_empty() {
            return 0.0;
        }

        let dimension = descriptors[0].len();
        let mut total_range = 0.0;

        for d in 0..dimension {
            let values: Vec<f32> = descriptors
                .iter()
                .map(|desc| desc.get(d).unwrap_or(&0.0))
                .copied()
                .collect();

            let min_val = values.iter().fold(f32::INFINITY, |a, &b| a.min(b));
            let max_val = values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
            total_range += max_val - min_val;
        }

        total_range / (dimension as f32 * 200.0)
    }
}
