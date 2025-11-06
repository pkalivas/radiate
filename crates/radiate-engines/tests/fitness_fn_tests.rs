use radiate_core::*;
use radiate_error::Result;

#[derive(Clone)]
pub struct FunctionDiversityProblem {
    pub codec: FloatCodec<Vec<f32>>,
    pub test_inputs: Vec<f32>,
}

impl FunctionDiversityProblem {
    pub fn new(codec: FloatCodec<Vec<f32>>, test_inputs: Vec<f32>) -> Self {
        Self { codec, test_inputs }
    }

    pub fn eval_raw(&self, weights: &Vec<f32>) -> f32 {
        let fitness: f32 = self
            .test_inputs
            .iter()
            .zip(weights.iter())
            .map(|(input, weight)| input * weight)
            .sum();

        (fitness - 44.0).abs() + 0.000001
    }
}

impl Problem<FloatChromosome, Vec<f32>> for FunctionDiversityProblem {
    fn encode(&self) -> Genotype<FloatChromosome> {
        self.codec.encode()
    }

    fn decode(&self, genotype: &Genotype<FloatChromosome>) -> Vec<f32> {
        self.codec.decode(genotype)
    }

    fn eval(&self, individual: &Genotype<FloatChromosome>) -> Result<Score> {
        let weights = self.decode(individual);
        Ok(Score::from(self.eval_raw(&weights)))
    }
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

#[cfg(test)]
mod fitness_fn_tests {
    use super::*;
    use radiate_engines::*;

    #[test]
    fn test_novelty_search() {
        random_provider::set_seed(42);

        let test_inputs = vec![4.0, -2.0, 3.5, 5.0, -11.0, -4.7];
        let codec = FloatCodec::vector(6, -100.0..100.0);

        let base_problem = FunctionDiversityProblem::new(codec.clone(), test_inputs.clone());

        let base_population = (0..100)
            .map(|_| Phenotype::from(base_problem.encode()))
            .collect::<Population<FloatChromosome>>();

        let regular_engine = GeneticEngine::builder()
            .problem(base_problem.clone())
            .population(base_population.clone())
            .survivor_selector(TournamentSelector::new(3))
            .offspring_selector(RouletteSelector::new())
            .alter(alters![
                UniformCrossover::new(0.7),
                GaussianMutator::new(0.2),
            ])
            .minimizing()
            .build();

        let novelty_engine = GeneticEngine::builder()
            .codec(codec.clone())
            .population(base_population.clone())
            .survivor_selector(TournamentSelector::new(3))
            .offspring_selector(RouletteSelector::new())
            .alter(alters![
                UniformCrossover::new(0.7),
                GaussianMutator::new(0.2),
            ])
            .fitness_fn(NoveltySearch::new(CosineDistance, 10, 0.03))
            .build();

        let cloned_base_problem = base_problem.clone();
        let combined_engine = GeneticEngine::builder()
            .codec(codec)
            .population(base_population.clone())
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
                        move |geno: &Vec<f32>| cloned_base_problem.eval_raw(geno),
                        0.7,
                    )
                    .add_weighted_fn(NoveltySearch::new(CosineDistance, 10, 0.03), 0.3),
            )
            .build();

        let regular_generation = regular_engine.iter().take(50).last().unwrap();
        let novelty_generation = novelty_engine.iter().take(50).last().unwrap();
        let combined_generation = combined_engine.iter().take(50).last().unwrap();

        let regular_diversity = calculate_diversity(regular_generation.population());
        let novelty_diversity = calculate_diversity(novelty_generation.population());
        let combined_diversity = calculate_diversity(combined_generation.population());

        let best_regular = regular_generation.value();
        let best_novelty = novelty_generation
            .population()
            .iter()
            .map(|ind| base_problem.decode(ind.genotype()))
            .min_by(|a, b| {
                base_problem
                    .eval_raw(&a)
                    .partial_cmp(&base_problem.eval_raw(&b))
                    .unwrap()
            })
            .unwrap();

        let best_combined = combined_generation
            .population()
            .iter()
            .map(|ind| base_problem.decode(ind.genotype()))
            .min_by(|a, b| {
                base_problem
                    .eval_raw(&a)
                    .partial_cmp(&base_problem.eval_raw(&b))
                    .unwrap()
            })
            .unwrap();

        let eval_regular = base_problem.eval_raw(best_regular);
        let eval_novelty = base_problem.eval_raw(&best_novelty);
        let eval_combined = base_problem.eval_raw(&best_combined);

        // 1. Novelty should have higher diversity than regular
        assert!(
            novelty_diversity > regular_diversity,
            "Novelty search should promote diversity: novelty={:.4}, regular={:.4}",
            novelty_diversity,
            regular_diversity
        );

        // 2. Combined should have higher diversity than regular
        assert!(
            combined_diversity > regular_diversity,
            "Combined approach should have higher diversity than regular: combined={:.4}, regular={:.4}",
            combined_diversity,
            regular_diversity
        );

        // 3. Novelty should have higher diversity than combined (pure novelty vs balanced)
        assert!(
            novelty_diversity > combined_diversity,
            "Pure novelty should have higher diversity than combined: novelty={:.4}, combined={:.4}",
            novelty_diversity,
            combined_diversity
        );

        // 4. Regular should have better fitness than novelty (novelty doesn't optimize fitness)
        assert!(
            eval_regular <= eval_novelty,
            "Regular should have better fitness than novelty: regular={:.4}, novelty={:.4}",
            eval_regular,
            eval_novelty
        );

        // 5. Combined should have better fitness than novelty (combines both objectives)
        assert!(
            eval_combined <= eval_novelty,
            "Combined should have better fitness than pure novelty: combined={:.4}, novelty={:.4}",
            eval_combined,
            eval_novelty
        );

        // 6. Regular should have better fitness than combined (combined sacrifices some fitness for diversity)
        assert!(
            eval_regular <= eval_combined,
            "Regular should have better fitness than combined: regular={:.4}, combined={:.4}",
            eval_regular,
            eval_combined
        );
    }
}
