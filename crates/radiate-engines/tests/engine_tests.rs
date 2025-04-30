#[cfg(test)]
mod engine_tests {
    use radiate_core::IntCodex;
    use radiate_engines::*;

    #[test]
    fn engine_can_minimize() {
        let codex = IntCodex::vector(5, 0..100);

        let engine = GeneticEngine::builder()
            .minimizing()
            .codex(codex)
            .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
            .build();

        let result = engine.iter().until_score_equal(0).last().unwrap();

        let best = result.result();
        assert_eq!(best.iter().sum::<i32>(), 0);
    }

    #[test]
    fn engine_can_maximize() {
        let codex = IntCodex::vector(5, 0..101);

        let engine = GeneticEngine::builder()
            .codex(codex)
            .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
            .build();

        let result = engine.iter().until_score_equal(500).last().unwrap();

        assert_eq!(result.result().iter().sum::<i32>(), 500);
    }

    #[test]
    fn engine_evolves_towards_target() {
        let target = [1, 2, 3, 4, 5];
        let codex = IntCodex::vector(target.len(), 0..10);

        let engine = GeneticEngine::builder()
            .minimizing()
            .codex(codex)
            .fitness_fn(move |geno: Vec<i32>| {
                let mut score = 0;
                for i in 0..geno.len() {
                    score += (geno[i] - target[i]).abs();
                }
                score
            })
            .build();

        let result = engine.iter().until_score_equal(0).last().unwrap();

        assert_eq!(result.result(), &vec![1, 2, 3, 4, 5]);
    }
}
