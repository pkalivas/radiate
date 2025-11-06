mod utils;

#[cfg(test)]
mod tests {
    use super::utils::*;
    use radiate_core::*;
    use std::sync::Arc;

    #[test]
    fn test_fitness_evaluator_eval() {
        let evaluator = FitnessEvaluator::new(Arc::new(Executor::Serial));
        let mut ecosystem = float_ecosystem();
        let problem = Arc::new(FloatEvalProblem);

        // All individuals should start without scores
        assert!(ecosystem.population.iter().all(|p| p.score().is_none()));

        let count = evaluator.eval(&mut ecosystem, problem).unwrap();

        // All individuals should now have scores
        assert!(ecosystem.population.iter().all(|p| p.score().is_some()));
        assert_eq!(count, 3);
    }

    #[test]
    fn test_batch_fitness_evaluator_eval() {
        let evaluator = BatchFitnessEvaluator::new(Arc::new(Executor::Serial));
        let mut ecosystem = float_ecosystem();
        let problem = Arc::new(FloatEvalProblem);

        // All individuals should start without scores
        assert!(ecosystem.population.iter().all(|p| p.score().is_none()));

        let count = evaluator.eval(&mut ecosystem, problem).unwrap();

        // All individuals should now have scores
        assert!(ecosystem.population.iter().all(|p| p.score().is_some()));
        assert_eq!(count, 3);
    }

    #[test]
    fn test_evaluator_trait_object() {
        let evaluator: Box<dyn Evaluator<FloatChromosome, f32>> =
            Box::new(FitnessEvaluator::new(Arc::new(Executor::Serial)));

        let mut ecosystem = float_ecosystem();
        let problem = Arc::new(FloatEvalProblem);

        let count = evaluator.eval(&mut ecosystem, problem).unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_empty_population() {
        let evaluator = FitnessEvaluator::new(Arc::new(Executor::Serial));
        let mut ecosystem = Ecosystem::<FloatChromosome>::default();
        let problem = Arc::new(FloatEvalProblem);

        let count = evaluator.eval(&mut ecosystem, problem).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_already_evaluated_population() {
        let evaluator = FitnessEvaluator::new(Arc::new(Executor::Serial));
        let mut ecosystem = float_ecosystem();
        let problem = Arc::new(FloatEvalProblem);

        // Pre-evaluate all individuals
        for phenotype in ecosystem.population_mut().iter_mut() {
            phenotype.set_score(Some(Score::from(1.0)));
        }

        let count = evaluator.eval(&mut ecosystem, problem).unwrap();
        assert_eq!(count, 0);
    }
}
