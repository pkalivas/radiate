mod utilities;

#[cfg(test)]
mod engine_tests {

    use radiate::*;

    #[test]
    fn engine_can_minimize() {
        let codex = IntCodex::new(1, 5, 0..100);

        let engine = GeneticEngine::from_codex(codex)
            .minimizing()
            .fitness_fn(|geno: Vec<Vec<i32>>| geno.iter().flatten().sum::<i32>())
            .build();

        let result = engine.run(|ctx| ctx.score().as_i32() == 0);

        let best = result.best.first().unwrap();
        assert_eq!(best.iter().sum::<i32>(), 0);
    }

    #[test]
    fn engine_can_maximize() {
        let codex = IntCodex::new(1, 5, 0..101);

        let engine = GeneticEngine::from_codex(codex)
            .fitness_fn(|geno: Vec<Vec<i32>>| geno.iter().flatten().sum::<i32>())
            .build();

        let result = engine.run(|ctx| ctx.score().as_i32() == 500);

        let best = result.best.first().unwrap();
        assert_eq!(best.iter().sum::<i32>(), 500);
    }

    #[test]
    fn engine_evolves_towards_target() {
        let target = [1, 2, 3, 4, 5];
        let codex = IntCodex::new(1, target.len(), 0..10);

        let engine = GeneticEngine::from_codex(codex)
            .minimizing()
            .fitness_fn(move |geno: Vec<Vec<i32>>| {
                let first = &geno[0];
                let mut score = 0;
                for i in 0..first.len() {
                    score += (first[i] - target[i]).abs();
                }
                score
            })
            .build();

        let result = engine.run(|ctx| ctx.score().as_i32() == 0);

        let best = result.best.first().unwrap();
        assert_eq!(best, &vec![1, 2, 3, 4, 5]);
    }
}
