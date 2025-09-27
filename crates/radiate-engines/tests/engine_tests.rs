#[cfg(test)]
mod engine_tests {
    use radiate_core::*;
    use radiate_engines::*;

    #[test]
    fn engine_can_minimize() {
        let engine = GeneticEngine::builder()
            .minimizing()
            .codec(IntCodec::vector(5, 0..100))
            .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
            .build();

        let result = engine.iter().until_score(0).last().unwrap();

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
    fn engine_can_eval_batch() {
        let mut engine = GeneticEngine::builder()
            .codec(IntChromosome::from((5, 0..100)))
            .minimizing()
            // The way the engine's workflow is setup, only individuals (phenotypes) which have been invalidated
            // (ie: those who have been mutated or crossed over) will be fed into the fitness function. Due to the
            // offspring fraction parameter being set at 0.8 by default, we'd expect a max of 80 individuals to be fed
            // into this batch fitness function, the rest will be copied down to the next generation via the survivor
            // selection. Although the real number will most likely be lower. To increase the number of individuals
            // fed into the batch we have two options:
            //   1.) Increase the offspring fraction as shown below (to 1.0) - this will cause the algorithm to completely
            //       negate the 'survivor_selector' and instead, will feed 100% of the population into the alters thus making
            //       every single phenotype open to crossover/mutation (invalidation - needing a new score).
            //          .offspring_fraction(1.0)
            //   2.) Increase the mutation/crossover rate so more individuals are invalidated during recombination.
            .executor(Executor::FixedSizedWorkerPool(7))
            .batch_fitness_fn(|phenotypes: &[Vec<i32>]| {
                // At a very very very base level, we expect the batch to have at least two phenotypes
                // Realistically, with an engine configured like this one is, we'd expect anywhere from 50-70ish
                // individuals per batch here.
                assert!(
                    phenotypes.len() > 1,
                    "Batch should have more than one phenotype"
                );
                phenotypes
                    .iter()
                    .map(|geno| geno.iter().sum::<i32>())
                    .collect()
            })
            .build();

        let result = engine.run(|ctx| ctx.score().as_i32() == 0);

        assert_eq!(result.value().iter().sum::<i32>(), 0);
    }

    #[test]
    fn test_engine_score_iterator() {
        let engine = GeneticEngine::builder()
            .minimizing()
            .codec(IntCodec::vector(5, 0..100))
            .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
            .build();

        let result = engine.iter().until_score(0).last().unwrap();

        let best = result.value();
        assert_eq!(best.iter().sum::<i32>(), 0);
        assert_eq!(result.score().as_i32(), 0);
    }

    #[test]
    fn test_engine_seconds_iterator() {
        let engine = GeneticEngine::builder()
            .minimizing()
            .codec(IntCodec::vector(5, 0..100))
            .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
            .build();

        let result = engine.iter().until_seconds(2_f64).last().unwrap();

        // Round here as the time taken to execute the engine may
        // be slightly over or under 2 seconds
        assert_eq!((result.time().as_secs_f64() - 2_f64).abs().round(), 0.0);
    }

    #[test]
    fn test_engine_iterations_iterator() {
        let engine = GeneticEngine::builder()
            .minimizing()
            .codec(IntCodec::vector(5, 0..100))
            .fitness_fn(|geno: Vec<i32>| geno.iter().sum::<i32>())
            .build();

        let result = engine.iter().limit(10).last().unwrap();
        assert_eq!(result.index(), 10);
    }
}
