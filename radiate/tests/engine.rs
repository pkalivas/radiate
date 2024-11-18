
#[cfg(test)]
mod engine_tests {

    use radiate::*;

    #[test]
    fn test_engine_can_minimize() {
        let codex = IntCodex::new(1, 5, 0, 100);

        let engine = GeneticEngine::from_codex(&codex)
            .minimizing()
            .fitness_fn(|genotype: Vec<Vec<i32>>| {
                Score::from_int(genotype.iter().fold(0, |acc, chromosome| {
                    acc + chromosome.iter().fold(0, |acc, gene| acc + gene)
                }))
            })
            .build();

        let result = engine.run(|output| output.score().as_int() == 0);

        let best = result.best.first().unwrap();
        assert_eq!(best.iter().fold(0, |acc, gene| acc + gene), 0);
    }

    #[test]
    fn test_engine_can_maximize() {
        let codex = IntCodex::new(1, 5, 0, 101);

        let engine = GeneticEngine::from_codex(&codex)
            .fitness_fn(|genotype: Vec<Vec<i32>>| {
                Score::from_int(genotype.iter().fold(0, |acc, chromosome| {
                    acc + chromosome.iter().fold(0, |acc, gene| acc + gene)
                }))
            })
            .build();

        let result = engine.run(|output| output.score().as_int() == 500);

        let best = result.best.first().unwrap();
        assert_eq!(best.iter().fold(0, |acc, gene| acc + gene), 500);
    }

    #[test]
    fn test_engine_evolves_towards_target() {
        let target = vec![1, 2, 3, 4, 5];
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
    fn test_population_initialization() {
        let codex = IntCodex::new(1, 5, 0, 100);
        let engine = GeneticEngine::from_codex(&codex)
            .population_size(150)
            .fitness_fn(|_| Score::from_int(1))
            .build();

        let result = engine.run(|_| true);
        assert_eq!(result.population.len(), 150);
        for individual in result.population {
            assert!(individual.is_valid());
        }
    }
}
