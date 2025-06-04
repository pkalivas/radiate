#[cfg(test)]
mod engine_tests {
    use radiate_core::IntCodec;
    use radiate_engines::*;

    #[test]
    fn engine_can_minimize() {
        let engine = GeneticEngine::builder()
            .minimizing()
            .codec(IntCodec::vector(5, 0..100))
            .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
            .build();

        let result = engine.iter().until_score_equal(0).unwrap();

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
}
