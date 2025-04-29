#[cfg(test)]
mod engine_tests {
    use radiate_core::IntCodex;
    use radiate_engines::*;

    #[test]
    fn engine_can_minimize() {
        let codex = IntCodex::vector(5, 0..100);

        let mut engine = GeneticEngine::from_codex(codex)
            .minimizing()
            .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
            .build();

        let result = engine.run(|ctx| ctx.score().as_i32() == 0);

        let best = result.result();
        assert_eq!(best.iter().sum::<i32>(), 0);
    }

    #[test]
    fn engine_can_maximize() {
        let codex = IntCodex::vector(5, 0..101);

        let mut engine = GeneticEngine::from_codex(codex)
            .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
            .build();

        let result = engine.run(|ctx| ctx.score().as_i32() == 500);

        assert_eq!(result.result().iter().sum::<i32>(), 500);
    }

    #[test]
    fn engine_evolves_towards_target() {
        let target = [1, 2, 3, 4, 5];
        let codex = IntCodex::vector(target.len(), 0..10);

        let mut engine = GeneticEngine::from_codex(codex)
            .minimizing()
            .fitness_fn(move |geno: Vec<i32>| {
                let mut score = 0;
                for i in 0..geno.len() {
                    score += (geno[i] - target[i]).abs();
                }
                score
            })
            .build();

        let result = engine.run(|ctx| ctx.score().as_i32() == 0);

        assert_eq!(result.result(), &vec![1, 2, 3, 4, 5]);
    }
}
